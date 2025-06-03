use clap::{ArgGroup, Parser};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

/// Simple HTTP client like curl
#[derive(Parser, Debug)]
#[command(name = "fetch")]
#[command(about = "A basic HTTP client CLI", long_about = None)]
#[command(group(
    ArgGroup::new("optional")
        .args(["method", "body", "header"])
        .multiple(true)
))]
struct Cli {
    /// Target host URL (e.g., https://example.com)
    #[arg(long)]
    host: String,

    /// HTTP method: GET, POST, PUT, DELETE (default: GET)
    #[arg(long, default_value = "GET")]
    method: String,

    /// HTTP body as a string (optional)
    #[arg(long)]
    body: Option<String>,

    /// Optional headers in the form "Key: Value" (can be repeated)
    #[arg(long)]
    header: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let method = cli.method.to_uppercase();
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    for h in cli.header {
        if let Some((k, v)) = h.split_once(":") {
            headers.insert(
                HeaderName::from_str(k.trim())?,
                HeaderValue::from_str(v.trim())?,
            );
        } else {
            eprintln!("Invalid header format: {}", h);
            return Ok(());
        }
    }

    let request_builder = match method.as_str() {
        "GET" => client.get(&cli.host),
        "POST" => client.post(&cli.host),
        "PUT" => client.put(&cli.host),
        "DELETE" => client.delete(&cli.host),
        _ => {
            eprintln!("Unsupported method: {}", method);
            return Ok(());
        }
    };

    let request = request_builder
        .headers(headers)
        .body(cli.body.unwrap_or_default())
        .build()?;

    let response = client.execute(request).await?;
    let status = response.status();
    let body = response.text().await?;

    println!("Status: {}", status);

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        let pretty = serde_json::to_string_pretty(&json)?;
        println!("Body:\n{}", pretty);
    } else {
        println!("Body:\n{}", body);
    }

    Ok(())
}
