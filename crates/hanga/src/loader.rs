use gyo_core::{GyoshoFile, Manifest, AssetKind};
use anyhow::{Context, Result};
use std::io::{Cursor, Read};
use binrw::BinRead; // <--- FIX: This was missing!

pub struct LoadedProject {
    pub manifest: Manifest,
    pub source_code: String,
}

pub struct ProjectLoader;

impl ProjectLoader {
    pub fn load(bytes: &[u8]) -> Result<LoadedProject> {
        // 1. Parse the Container
        let mut cursor = Cursor::new(bytes);
        // This .read() call requires binrw::BinRead to be in scope
        let file = GyoshoFile::read(&mut cursor)
            .context("Failed to parse GYO header")?;
        
        let manifest: Manifest = bincode::deserialize(&file.manifest_bytes)
            .context("Failed to deserialize Manifest")?;

        // 2. Decompress Payload
        let mut decoder = zstd::Decoder::new(Cursor::new(&file.compressed_payload))?;
        let mut decompressed_payload = Vec::new();
        decoder.read_to_end(&mut decompressed_payload)
            .context("Failed to decompress GYO payload")?;

        // 3. Extract Source Code
        let source_code_asset = manifest.assets.iter()
            .find(|a| matches!(a.kind, AssetKind::SumiSource))
            .context("No Source Code found in project manifest")?;

        let start = source_code_asset.offset as usize;
        let end = start + source_code_asset.size as usize;
        
        if end > decompressed_payload.len() {
            anyhow::bail!("Corrupt file: Asset defined outside payload bounds");
        }

        let source_bytes = &decompressed_payload[start..end];
        let source_code = String::from_utf8(source_bytes.to_vec())
            .context("Source code is not valid UTF-8")?;

        Ok(LoadedProject {
            manifest,
            source_code,
        })
    }
}
