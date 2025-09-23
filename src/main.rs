use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Parser;
use flate2::{Compression, write::GzEncoder};

mod cli;
mod overpass;
mod utils;

use cli::Args;
use overpass::{download_data, parse_response, process_elements};
use utils::{get_file_size_human, log_debug, log_error, log_info};

/// Extra seconds to add to HTTP client timeout beyond the Overpass query timeout
/// This provides buffer for network latency and response processing
const HTTP_TIMEOUT_BUFFER_SECONDS: u64 = 1;

fn serialize_json_pretty<T: serde::Serialize>(value: &T, indent: &[u8]) -> Result<Vec<u8>> {
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent);
    let mut output = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut output, formatter);
    value
        .serialize(&mut serializer)
        .context("Failed to serialize to JSON")?;
    Ok(output)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Build HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            args.timeout_seconds as u64 + HTTP_TIMEOUT_BUFFER_SECONDS,
        ))
        .build()
        .context("Failed to build HTTP client")?;

    // Step 1: Download data
    let overpass_api_endpoint = args.get_overpass_url();
    log_info(&format!(
        "1: Downloading data through Overpass API (this may take up to {} seconds...)",
        args.timeout_seconds
    ));
    log_debug(&format!(
        "-> Using overpass API endpoint: {}",
        overpass_api_endpoint
    ));

    let response_bytes =
        download_data(&client, overpass_api_endpoint, args.timeout_seconds).await?;

    // Parse the JSON response
    let overpass_response = parse_response(&response_bytes)?;

    let element_count = overpass_response.elements.len();
    if element_count == 0 {
        log_error("Query failed, found 0 elements.");
        // Try to parse for error details
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            && let Some(remark) = json_value.get("remark").and_then(|v| v.as_str())
        {
            log_error(&format!("Details: {}", remark));
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

    let processed = process_elements(overpass_response.elements)?;

    // Serialize to JSON with pretty formatting (1-space indentation)
    let json_output = serialize_json_pretty(&processed, b" ")?;

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
