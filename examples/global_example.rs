use clap::Parser;
use ff_auth::prelude::*;
use tracing::{info, error, Level};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    username: String,
    
    #[arg(short, long)]
    password: String,
    
    #[arg(short, long)]
    otp: Option<String>,
    
    #[arg(short, long)]
    region: Option<i8>,
    
    #[arg(long)]
    free_trial: bool,
    
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Parse log level
    let level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("Starting FF Auth Global Example");
    info!("Username: {}", args.username);
    info!("Password: {}", args.password);
    info!("Region: {:?}", args.region);
    info!("Free trial: {}", args.free_trial);
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Build login request
    let mut login_request = LoginRequest::new(client)
        .with_username(args.username)
        .with_password(args.password);
    
    if let Some(otp) = args.otp {
        login_request = login_request.with_otp(otp);
    }
    
    if let Some(region) = args.region {
        login_request = login_request.with_region(region);
    }
    
    if args.free_trial {
        login_request = login_request.with_free_trial();
    }
    
    info!("Login request: {:?}", login_request);
    
    // Create global client
    let global_client = GlobalClient::default();
    
    // Attempt authentication
    match global_client.authenticate(login_request).await {
        Ok(response) => {
            info!("Authentication successful!");
            info!("Response: {:?}", response);
        }
        Err(e) => {
            error!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}