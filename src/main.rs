use crate::bundles::BundleVersion;
use serde::{Deserialize, Serialize};
use std::error::Error;

mod bundles;

use clap::Parser;
use cli_colors::Colorizer;
use itertools::Itertools;


#[derive(Parser)]
#[command(version, about = "Appropos: Software Version Checker")]
struct Args {
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

    let meh = results
        .into_iter()
        .flatten()
        .collect::<Vec<BundleVersion>>();

    let resp: Vec<Recommendation> = client
        .post("https://api.appropos.app/check")
        // .post("http://localhost:8080/check")
        .json(&meh)
        .send()
        .await?
        .json()
        .await?;

    let colorizer = Colorizer::new();

    print_bundles(colorizer.underline(colorizer.red("Recommended Updates:")), resp
        .iter()
        .filter(|b| b.recommendation_type == Some(String::from("UPDATE"))).collect_vec());

    print_bundles(colorizer.underline(colorizer.green("Upto Date:")), resp
        .iter()
        .filter(|b| b.recommendation_type.is_none()).collect_vec());

    print_bundles(colorizer.underline(colorizer.green("Untracked by Appropos")), resp
        .iter()
        .filter(|b| b.recommendation_type == Some(String::from("UNTRACKED"))).collect_vec());

    Ok(())
}

fn print_bundles(section_title: String, bundles: Vec<&Recommendation>) {
    println!("{} - ({})", section_title, bundles.len());
    if bundles.is_empty() {
        println!("None");
    } else {
        for bundle in bundles {
            print_bundle(&bundle);
        }
    }
    println!();
}

fn print_bundle(bundle: &Recommendation) {
    println!("{} - {} - {}", bundle.name, bundle.id, bundle.version, );
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
