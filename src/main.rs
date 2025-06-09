use clap::{ArgGroup, Parser};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct RequestFile {
    url: String,
    method: String,
    body: Option<serde_json::Value>,
    headers: Option<Vec<String>>,
}

/// Simple HTTP client like curl
#[derive(Parser, Debug)]
#[command(name = "fetch")]
#[command(about = "A basic HTTP client CLI", long_about = None)]
#[command(group(
    ArgGroup::new("input")
        .args(["file", "host"])
        .required(true)
))]
struct Cli {
    /// Path to JSON file describing the request
    #[arg(long)]
    file: Option<String>,

    /// Target host URL (e.g., https://example.com)
    #[arg(long)]
    host: Option<String>,

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

    let (url, method, body_opt, headers_vec) = if let Some(file_path) = cli.file {
        let file_content = std::fs::read_to_string(file_path)?;
        let parsed: RequestFile = serde_json::from_str(&file_content)?;
        (
            parsed.url,
            parsed.method.to_uppercase(),
            parsed.body.map(|v| v.to_string()),
            parsed.headers.unwrap_or_default(),
        )
    } else {
        (
            cli.host.expect("host is required if file is not provided"),
            cli.method.to_uppercase(),
            cli.body,
            cli.header,
        )
    };

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    for h in headers_vec {
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
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => {
            eprintln!("Unsupported method: {}", method);
            return Ok(());
        }
    };

    let request = request_builder
        .headers(headers)
        .body(body_opt.unwrap_or_default())
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
