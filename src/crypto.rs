use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

pub fn compute_sha1(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let mut hasher = Sha1::new();

    if meta.is_file() && (1..=(1 << 22)).contains(&meta.len()) {
        let mut buf = vec![0u8; meta.len() as usize];
        file.read_exact(&mut buf)?;
        hasher.update(&buf);
    } else {
        let mut chunk = vec![0u8; 1 << 22];
        loop {
            match file.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => hasher.update(&chunk[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    let digest = hasher.finalize();
    let mut out = String::with_capacity(40);
    for b in digest {
        out.push(HEX_CHARS[(b >> 4) as usize] as char);
        out.push(HEX_CHARS[(b & 0xf) as usize] as char);
    }
    Ok(out)
}
