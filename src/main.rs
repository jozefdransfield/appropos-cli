use crate::bundles::BundleVersion;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::FileType;

mod bundles;

use clap::Parser;
use itertools::Itertools;

#[derive(Parser)]
#[command(version, about = "Appropos: Software Version Checker")]
struct Args {
    /// Output as JSON
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let results = bundles::list();

    if results.is_empty() {
        eprintln!("apropos: nothing appropriate.");
        std::process::exit(1);
    }

    if args.verbose {
        for result in &results {
            match result {
                Ok(bundle) => {
                    let x = bundle
                        .meta
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .join(", ");
                    println!(
                        "{} - {} - {} - {} - {}",
                        bundle.name, bundle.id, bundle.source, bundle.version, x
                    );
                }
                Err(e) => println!("apropos: failed to parse plist: {}", e),
            }
        }
        println!("\n\n");
    }

    let client = reqwest::Client::new();

    let meh = results.into_iter().flatten().collect::<Vec<BundleVersion>>();

    let resp: Vec<Recommendation> = client
        .post("https://api.appropos.app/check")
        .json(&meh)
        .send().await?
        .json().await?;

    for bundle in resp {
        let recommendation = bundle.recommendation_type.unwrap_or("".to_string());
        // if (recommendation == "UPDATE") {
        println!(
            "{} - {} - {} - {} - {}",
            bundle.name,
            bundle.id,
            bundle.version,
            bundle.recommended_version.unwrap_or("".to_string()),
            recommendation
        );
        // }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Recommendation {
    pub name: String,
    pub id: String,
    pub version: String,
    #[serde(rename = "recommendedVersion")]
    pub recommended_version: Option<String>,
    #[serde(rename = "type")]
    pub recommendation_type: Option<String>,
}
