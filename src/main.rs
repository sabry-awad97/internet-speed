mod core;
mod services;
mod utils;

use crate::services::download_tester::DownloadTester;
use crate::services::speed_service::SpeedService;
use crate::services::upload_tester::UploadTester;
use crate::utils::http_client::ReqwestHttpClient;
use clap::Parser;
use std::sync::Arc;

/// Maximum upload size in bytes (512 MB) to prevent overflow and excessive memory allocation
const MAX_UPLOAD_BYTES: usize = 512 * 1024 * 1024;

#[derive(Parser, Debug)]
#[command(author, version, about = "Internet speed tester (Rust)")]
struct Args {
    /// Download test URL (must support streaming large file)
    #[arg(long, default_value = "https://proof.ovh.net/files/10Mb.dat")]
    download_url: String,

    /// Upload endpoint URL (should accept POST)
    #[arg(long, default_value = "https://httpbin.org/post")]
    upload_url: String,

    /// Upload size in megabytes (max 512 MB)
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=512))]
    upload_mb: u32,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("=== Internet Speed Test ===");
    println!("Download URL: {}", args.download_url);
    println!("Upload URL: {}", args.upload_url);
    println!("Upload size: {} MB\n", args.upload_mb);

    let client = Arc::new(ReqwestHttpClient::new());

    // Create testers with dependency injection of HttpClient trait object
    let download =
        Arc::new(DownloadTester::new(client.clone(), args.download_url).with_max_bytes(10_000_000));
    // Convert MB to bytes with overflow protection
    let upload_bytes = (args.upload_mb as usize)
        .saturating_mul(1024)
        .saturating_mul(1024)
        .min(MAX_UPLOAD_BYTES);

    let upload = Arc::new(UploadTester::new(
        client.clone(),
        args.upload_url,
        upload_bytes,
    ));

    let service = SpeedService::new(download, upload);

    match service.run().await {
        Ok(res) => {
            println!("\n=== Results ===");
            println!("Download: {:.2} Mbps", res.download_mbps);
            println!("Upload:   {:.2} Mbps", res.upload_mbps);
            Ok(())
        }
        Err(e) => {
            eprintln!("Speed test failed: {}", e);
            Err(anyhow::anyhow!(e))
        }
    }
}
