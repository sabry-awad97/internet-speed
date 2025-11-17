mod core;
mod services;
mod utils;

use crate::services::download_tester::DownloadTester;
use crate::services::speed_service::SpeedService;
use crate::services::upload_tester::UploadTester;
use crate::utils::http_client::ReqwestHttpClient;
use clap::Parser;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about = "Internet speed tester (Rust)")]
struct Args {
    /// Download test URL (must support streaming large file)
    #[arg(long, default_value = "https://proof.ovh.net/files/10Mb.dat")]
    download_url: String,

    /// Upload endpoint URL (should accept POST)
    #[arg(long, default_value = "https://httpbin.org/post")]
    upload_url: String,

    /// Upload size in megabytes
    #[arg(long, default_value_t = 1)]
    upload_mb: usize,
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
    let upload = Arc::new(UploadTester::new(
        client.clone(),
        args.upload_url,
        args.upload_mb * 1024 * 1024,
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
