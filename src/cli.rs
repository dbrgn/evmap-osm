use clap::Parser;

#[derive(Debug, Clone)]
pub enum OverpassApiEndpoint {
    /// Use the API from overpass.osm.ch (which only includes Switzerland, good for quick testing)
    Switzerland,
    /// Use the API from overpass-api.de (which includes the whole world)
    World,
    /// Use a custom Overpass API URL
    Custom(String),
}

impl OverpassApiEndpoint {
    /// Parse a string into an OverpassApiScope
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "switzerland" => Ok(OverpassApiEndpoint::Switzerland),
            "world" => Ok(OverpassApiEndpoint::World),
            url if url.starts_with("http://") || url.starts_with("https://") => {
                Ok(OverpassApiEndpoint::Custom(url.to_string()))
            }
            _ => Err(format!(
                "Invalid value '{}'. Expected 'switzerland', 'world', or a URL starting with http:// or https://",
                s
            )),
        }
    }
}

impl OverpassApiEndpoint {
    /// Get the Overpass API URL for this scope
    pub fn get_url(&self) -> &str {
        match self {
            OverpassApiEndpoint::Switzerland => "https://overpass.osm.ch/api/interpreter",
            OverpassApiEndpoint::World => "https://overpass-api.de/api/interpreter",
            OverpassApiEndpoint::Custom(url) => url,
        }
    }
}

/// Download worldwide charging station data from Overpass API and write them to
/// a compressed JSON file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Keep intermediate raw response file
    #[arg(long, value_name = "BOOL", default_value = "false")]
    pub keep_intermediate: bool,

    /// Overpass API to use ('switzerland', 'world', or a custom URL)
    ///
    /// The following hardcoded endpoints are supported:
    ///
    /// switzerland (https://overpass.osm.ch/api/interpreter),
    /// world (https://overpass-api.de/api/interpreter)
    #[arg(long, value_parser = OverpassApiEndpoint::parse)]
    pub overpass_api_endpoint: OverpassApiEndpoint,

    /// Timeout in seconds for the Overpass query
    #[arg(long, default_value = "900")]
    pub timeout_seconds: u32,

    /// Output file for raw response (only written if --keep-intermediate=true)
    #[arg(long, default_value = "overpass-result.json")]
    pub outfile_raw: String,

    /// Output file for compressed result
    #[arg(long, default_value = "charging-stations-osm.json.gz")]
    pub outfile_compressed: String,
}

impl Args {
    /// Get the appropriate Overpass API URL based on the scope
    pub fn get_overpass_url(&self) -> &str {
        self.overpass_api_endpoint.get_url()
    }
}
