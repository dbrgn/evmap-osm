use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Overpass API response structure
#[derive(Debug, Deserialize)]
pub struct OverpassResponse {
    #[allow(dead_code)]
    version: f64,
    #[allow(dead_code)]
    generator: String,
    pub elements: Vec<Element>,
}

/// Element in the Overpass response
#[derive(Debug, Deserialize, Serialize)]
pub struct Element {
    #[serde(rename = "type")]
    pub element_type: String,
    pub id: u64,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    pub timestamp: String,
    pub version: u32,
    #[serde(default)]
    pub changeset: Option<u64>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub uid: Option<u64>,
    pub tags: HashMap<String, String>,
}

/// Processed output structure
#[derive(Debug, Serialize)]
pub struct ProcessedOutput {
    pub timestamp: u64,
    pub count: usize,
    pub elements: Vec<ProcessedElement>,
}

/// Processed element with only required fields
#[derive(Debug, Serialize)]
pub struct ProcessedElement {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    pub tags: HashMap<String, String>,
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

/// Build the Overpass QL query for charging stations
fn build_query(timeout_seconds: u32) -> String {
    format!(
        "
        [out:json][timeout:{}];
        (
          node[amenity=charging_station];
          area[amenity=charging_station];
          relation[amenity=charging_station];
        );
        out meta qt;
        ",
        timeout_seconds
    )
}

/// Download data from Overpass API
pub async fn download_data(
    client: &reqwest::Client,
    url: &str,
    timeout_seconds: u32,
) -> Result<Bytes> {
    let query = build_query(timeout_seconds);

    let response = client
        .post(url)
        .header("content-type", "text/plain")
        .body(query)
        .send()
        .await
        .context("Failed to send request to Overpass API")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP request failed with status: {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;
    Ok(bytes)
}

/// Parse Overpass response from bytes
pub fn parse_response(response_bytes: &[u8]) -> Result<OverpassResponse> {
    serde_json::from_slice(response_bytes).context("Failed to parse Overpass API response as JSON")
}

/// Process elements into the output format
pub fn process_elements(elements: Vec<Element>) -> Result<ProcessedOutput> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_secs();

    Ok(ProcessedOutput {
        timestamp,
        count: elements.len(),
        elements: elements.into_iter().map(ProcessedElement::from).collect(),
    })
}
