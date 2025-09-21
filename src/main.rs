use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Parser;
use flate2::{Compression, write::GzEncoder};

mod cli;
mod overpass;

use cli::Args;
use overpass::{download_data, parse_response, process_elements};

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

    // Build HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            args.timeout_seconds as u64 + 30,
        ))
        .build()
        .context("Failed to build HTTP client")?;

    // Step 1: Download data
    log_info(&format!(
        "1: Downloading data through Overpass API (this may take up to {} seconds...)",
        args.timeout_seconds
    ));

    let response_bytes = download_data(&client, &args.overpass_url, args.timeout_seconds).await?;

    // Parse the JSON response
    let overpass_response = parse_response(&response_bytes)?;

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

    let processed = process_elements(overpass_response.elements)?;

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
