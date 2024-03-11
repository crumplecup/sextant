use sextant::prelude::*;
use clap::Parser;
use polite::Polite;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Polite<()> {
    let cli = Cli::parse();
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cordial=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init().is_ok() {};
    info!("Subscriber initialized.");

    match cli.command.as_str() {
        "read" => {
            info!("Loading parcels from shapefile.");
            cli.load_parcels()?;
            info!("Parcels loaded.");
        }
        "load" => {
            info!("Loading parcels from file.");
            Cli::load()?;
            info!("Parcels loaded.");
        },
        "transform" => {
            info!("Loading parcels.");
            cli.transform()?;
            info!("Parcels saved.");
        }
        "save" => {
            info!("Saving parcels to binary.");
            cli.save_parcels()?;
            info!("Parcels saved.");
        },
        "viewer" => {
            info!("Loading viewer.");
            Viewer::run(&cli).await?;
            info!("Viewer closed.");
        }
        _ => {
            info!("Unrecognized command.  See --help.");
        },
    }

    Ok(())
}
