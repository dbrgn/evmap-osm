use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use clap::Parser;
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};

/// Download worldwide charging station data from Overpass API and write them to
/// a compressed JSON file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Keep intermediate raw response file
    #[arg(long, value_name = "BOOL", default_value = "false")]
    keep_intermediate: bool,

    /// Overpass API interpreter URL
    #[arg(long, default_value = "https://overpass.osm.ch/api/interpreter")]
    overpass_url: String,

    /// Timeout in seconds for the Overpass query
    #[arg(long, default_value = "900")]
    timeout_seconds: u32,

    /// Output file for raw response (only written if --keep-intermediate=true)
    #[arg(long, default_value = "overpass-result.json")]
    outfile_raw: String,

    /// Output file for compressed result
    #[arg(long, default_value = "charging-stations-osm.json.gz")]
    outfile_compressed: String,
}

/// Overpass API response structure
#[derive(Debug, Deserialize)]
struct OverpassResponse {
    #[allow(dead_code)]
    version: f64,
    #[allow(dead_code)]
    generator: String,
    elements: Vec<Element>,
}

/// Element in the Overpass response
#[derive(Debug, Deserialize, Serialize)]
struct Element {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lon: Option<f64>,
    timestamp: String,
    version: u32,
    #[serde(default)]
    changeset: Option<u64>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    uid: Option<u64>,
    tags: HashMap<String, String>,
}

/// Processed output structure
#[derive(Debug, Serialize)]
struct ProcessedOutput {
    timestamp: u64,
    count: usize,
    elements: Vec<ProcessedElement>,
}

/// Processed element with only required fields
#[derive(Debug, Serialize)]
struct ProcessedElement {
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lon: Option<f64>,
    timestamp: String,
    #[serde(rename = "type")]
    element_type: String,
    version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    tags: HashMap<String, String>,
}

impl From<Element> for ProcessedElement {
    fn from(elem: Element) -> Self {
        ProcessedElement {
            id: elem.id,
            lat: elem.lat,
            lon: elem.lon,
            timestamp: elem.timestamp,
            element_type: elem.element_type,
            version: elem.version,
            user: elem.user,
            tags: elem.tags,
        }
    }
}

fn log_info(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

fn log_error(msg: &str) {
    eprintln!("\x1b[31m{}\x1b[0m", msg);
}

fn get_file_size_human(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len() as f64;
            if size < 1024.0 {
                format!("{:.0}B", size)
            } else if size < 1024.0 * 1024.0 {
                format!("{:.1}K", size / 1024.0)
            } else if size < 1024.0 * 1024.0 * 1024.0 {
                format!("{:.1}M", size / (1024.0 * 1024.0))
            } else {
                format!("{:.1}G", size / (1024.0 * 1024.0 * 1024.0))
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Build the Overpass query
    let query = format!(
        "[out:json][timeout:{}]; ( node[amenity=charging_station]; area[amenity=charging_station]; relation[amenity=charging_station]; ); out meta qt;",
        args.timeout_seconds
    );

    // Step 1: Download data
    log_info(&format!(
        "1: Downloading data through Overpass API (this may take up to {} seconds...)",
        args.timeout_seconds
    ));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            args.timeout_seconds as u64 + 30,
        ))
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .post(&args.overpass_url)
        .header("content-type", "text/plain")
        .body(query)
        .send()
        .await
        .context("Failed to send request to Overpass API")?;

    if !response.status().is_success() {
        log_error(&format!(
            "HTTP request failed with status: {}",
            response.status()
        ));
        anyhow::bail!("HTTP request failed with status: {}", response.status());
    }

    let response_bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    // Parse the JSON response
    let overpass_response: OverpassResponse = serde_json::from_slice(&response_bytes)
        .context("Failed to parse Overpass API response as JSON")?;

    let element_count = overpass_response.elements.len();
    if element_count == 0 {
        log_error("Query failed, found 0 elements.");
        // Try to parse for error details
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&response_bytes) {
            if let Some(remark) = json_value.get("remark").and_then(|v| v.as_str()) {
                log_error(&format!("Details: {}", remark));
            }
        }
        anyhow::bail!("No elements found in response");
    }

    // Save intermediate file if requested
    if args.keep_intermediate {
        std::fs::write(&args.outfile_raw, &response_bytes)
            .context("Failed to write intermediate file")?;
        let size_raw = get_file_size_human(Path::new(&args.outfile_raw));
        log_info(&format!(
            "Saved intermediate file: {} ({})",
            args.outfile_raw, size_raw
        ));
    }

    // Step 2: Process the data
    log_info(&format!("2: Processing {} entries", element_count));

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_secs();

    // Create processed output
    let processed = ProcessedOutput {
        timestamp,
        count: element_count,
        elements: overpass_response
            .elements
            .into_iter()
            .map(ProcessedElement::from)
            .collect(),
    };

    // Serialize to JSON
    let json_output =
        serde_json::to_vec(&processed).context("Failed to serialize processed data to JSON")?;

    // Compress with gzip
    let output_file = File::create(&args.outfile_compressed)
        .with_context(|| format!("Failed to create output file: {}", args.outfile_compressed))?;
    let mut encoder = GzEncoder::new(output_file, Compression::best());
    encoder
        .write_all(&json_output)
        .context("Failed to write compressed data")?;
    encoder
        .finish()
        .context("Failed to finish gzip compression")?;

    let size_compressed = get_file_size_human(Path::new(&args.outfile_compressed));
    log_info(&format!(
        "Done: {} ({})",
        args.outfile_compressed, size_compressed
    ));

    Ok(())
}
