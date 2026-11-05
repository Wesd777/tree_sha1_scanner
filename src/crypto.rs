use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// Helper function to compute the SHA-1 checksum hex string using sha1
pub fn compute_sha1(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let digest = hasher.finalize();
    let hex_string = digest.iter().map(|b| format!("{:02x}", b)).collect();

    Ok(hex_string)
}

