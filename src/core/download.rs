use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

pub struct Downloader;

impl Default for Downloader {
    fn default() -> Self {
        Self
    }
}

impl Downloader {
    pub fn new() -> Self {
        Self
    }

    pub fn download_file(&self, url: &str, destination: &Path) -> Result<()> {
        println!("Downloading from {url}...");

        // Ensure parent directory exists
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = std::process::Command::new("curl")
            .arg("-L") // Follow redirects
            .arg("-s") // Silent
            .arg("-H")
            .arg("User-Agent: cleen/0.1.0")
            .arg("-o")
            .arg(destination)
            .arg(url)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to download file: curl exited with status {:?}",
                output.status.code()
            ));
        }

        println!("Downloaded to {destination:?}");
        Ok(())
    }

    pub fn extract_archive(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        println!("Extracting {archive_path:?} to {destination:?}");

        std::fs::create_dir_all(destination)?;

        let file_name = archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid archive file name"))?;

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            self.extract_tar_gz(archive_path, destination)?;
        } else if file_name.ends_with(".zip") {
            self.extract_zip(archive_path, destination)?;
        } else {
            return Err(anyhow::anyhow!("Unsupported archive format: {}", file_name));
        }

        println!("Extraction completed");
        Ok(())
    }

    fn extract_tar_gz(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(destination)?;
        Ok(())
    }

    fn extract_zip(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => destination.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }
        }
        Ok(())
    }
}
