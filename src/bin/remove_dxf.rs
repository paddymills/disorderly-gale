
//! Remove DXF files
//! 
//! This takes care of the problem of DXF files that are used as an
//! intermediate format between NX and Sigmanest taking up too much
//! server space. These files are used so that we do not version
//! lock the two softwares.
//! 
//! This utility, broadly speaking, finds any file matching the glob
//! pattern `\\hssieng\Jobs\**\Fab\**\DXF\*.dxf` where
//! - The file is older than 60 days
//! - The file also has an associated `.log` file
//! 
//! This ensures that we are not deleted DXF files that are not yet
//! imported, as well as ones that did not originate from NX (generally,
//! only the NX generated DXF's will have an associated `.log` file.
//! The found files are then deleted. 


use std::error::Error;
use std::{fs, sync::OnceLock};
use std::path::Path;
use std::time::Duration;

use wax::{Glob, FileIterator};

const ROOT_DIR: &str = r"\\hssieng\Jobs";
const SIXTY_DAYS: Duration = Duration::from_secs(60 * 24 * 60 * 60);  // days * hours * minutes * seconds
static DXF_FILES: OnceLock<Glob> = OnceLock::new();

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    DXF_FILES.set( Glob::new("*.dxf")? ).expect("Failed to set `DXF_FILES` Glob pattern");

    // walk DXF folders so that we can filter out folders based on last modified
    let deleted: u32 = Glob::new("**/Fab/**/DXF")?
        .walk(Path::new(ROOT_DIR))
        .filter_tree(filter_dxf_folders)
        .filter_map(|dir| dir.ok())
        .map(|entry| remove_files(entry.path()))
        .sum();

    log::info!("Deleted {} dxf files", deleted);

    Ok(())
}

fn filter_dxf_folders(entry: &wax::WalkEntry) -> Option<wax::FilterTarget> {
    let too_old = |metadata: Result<fs::Metadata, wax::WalkError>| -> Result<bool, Box<dyn Error>> {
        Ok( metadata?.modified()?.elapsed()? < SIXTY_DAYS )
    };

    // we only want directories named `DXF`
    if !entry.path().is_dir() {
        log::debug!("Skipping non-dir `{}`", entry.path().display());
        Some(wax::FilterTarget::File)   // Filter out file
    }
    
    // filter out folders with modified date > SIXTY_DAYS
    else if too_old(entry.metadata()).ok()? {
        log::debug!("Skipping entry `{}` (last modified less than 60 days ago)", entry.path().display());
        Some(wax::FilterTarget::Tree)   // filter out directory
    }
    
    else { None }
}

fn remove_files(path: &Path) -> u32 {
    log::debug!("Walking directory {}", path.display());

    DXF_FILES.get().unwrap().walk(path)
        .filter_map(|e| e.ok())
        .map(|entry| remove_file(entry.path()).is_ok() as u32)
        .sum()
}

/// Recursively remove 
#[allow(dead_code)]
fn remove_file(path: &Path) -> Result<(), std::io::Error> {
    log::debug!("Removing .dxf/.log file {}", path.display());
        
    // File is older than 60 days
    fs::remove_file(&path)?;

    // Also remove the corresponding .log file
    fs::remove_file(&path.with_extension("log"))?;

    Ok(())
}
