use clap::Parser;

/// Download worldwide charging station data from Overpass API and write them to
/// a compressed JSON file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Keep intermediate raw response file
    #[arg(long, value_name = "BOOL", default_value = "false")]
    pub keep_intermediate: bool,

    /// Overpass API interpreter URL
    #[arg(long, default_value = "https://overpass.osm.ch/api/interpreter")]
    pub overpass_url: String,

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
