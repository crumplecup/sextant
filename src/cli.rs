use crate::prelude::Parcels;
use polite::{Polite, FauxPas};
use clap::Parser;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'c', long, help = "Command to execute.")]
    pub command: String,
    #[arg(short = 'i', long, help = "Input path.", default_value = None, default_missing_value = None)]
    pub input: Option<std::path::PathBuf>,
    #[arg(short = 'o', long, help = "Output path.", default_value = None, default_missing_value = None)]
    pub output: Option<std::path::PathBuf>,
}

impl Cli {
    pub fn load() -> Polite<Parcels> {
        let parcels: Parcels = bincode::deserialize(include_bytes!("../data/parcels.data"))?;
        info!("Parcels read: {}.", parcels.records.len());
        Ok(parcels)
    }

    pub fn load_parcels(&self) -> Polite<Parcels> {
        if let Some(path) = self.input.clone() {
            let parcels = Parcels::from_shp(path, None)?;
            info!("Parcels read: {}.", parcels.records.len());
            Ok(parcels)
        } else {
            Err(FauxPas::Unknown)
        }
    }

    pub fn transform(&self) -> Polite<()> {
        if let Some(path) = self.input.clone() {
            let parcels = Parcels::from_shp(path, Some("EPSG:2270"))?;
            info!("Parcels read: {}.", parcels.records.len());
            if let Some(out) = self.output.clone() {
                parcels.save(out)?;
                Ok(())
            } else {
                info!("Output path missing.");
                Err(FauxPas::Unknown)
            }
        } else {
            info!("Input path missing.");
            Err(FauxPas::Unknown)
        }
    }

    pub fn save_parcels(&self) -> Polite<()> {
        if let Some(path) = self.output.clone() {
            let parcels = self.load_parcels()?;
            parcels.save(path)?;
            Ok(())
        } else {
            Err(FauxPas::Unknown)
        }
    }
}

