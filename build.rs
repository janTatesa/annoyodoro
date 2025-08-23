use std::{error::Error, fs, path::PathBuf};

use isahc::ReadResponseExt;
const VERSION: &str = "0.541.0";
fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(format!("target/Lucide-{VERSION}.ttf"));
    if !path.exists() {
        let bytes = isahc::get(format!(
            "https://unpkg.com/lucide-static@{VERSION}/font/Lucide.ttf"
        ))?
        .bytes()?;
        fs::write(path, bytes)?;
    }

    Ok(())
}
