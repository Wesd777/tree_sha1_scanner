use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// Helper function to compute the SHA-1 checksum hex string using sha1
pub fn compute_sha1(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    // Optimized internal buffer capacity to match OS page boundaries
    let mut reader = BufReader::with_capacity(65536, file);
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let digest = hasher.finalize();

    // Performance fix: Allocate exactly 40 chars for SHA-1 to eliminate heap reallocations
    let mut hex_string = String::with_capacity(40);
    for byte in digest {
        hex_string.push_str(&format!("{:02x}", byte));
    }

    Ok(hex_string)
}
