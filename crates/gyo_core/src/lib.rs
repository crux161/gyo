use binrw::{binrw, BinRead, BinWrite};
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};

/// The Magic Signature: "GYO1"
const MAGIC: &[u8; 4] = b"GYO1";

#[binrw]
#[brw(big)] // Network Endian (Big Endian) for portability
#[derive(Debug, Clone)]
pub struct GyoshoFile {
    /// Magic Bytes to identify the file format
    #[br(assert(magic == *MAGIC))]
    pub magic: [u8; 4],

    /// Schema Version
    pub version: u32,

    /// The Manifest Size in bytes. 
    /// We read this first so we know how much JSON metadata to parse.
    pub manifest_len: u32,

    /// The Manifest: Describes the scene, assets, and logic.
    /// It is stored as raw bytes here, but typically serialized JSON.
    #[br(count = manifest_len)]
    pub manifest_bytes: Vec<u8>,

    /// The compressed payload follows immediately after.
    #[br(parse_with = binrw::helpers::until_eof)]
    pub compressed_payload: Vec<u8>,
}

/// The "Table of Contents" for the project.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub title: String,
    pub author: String,
    pub timestamp: u64,
    pub assets: Vec<AssetEntry>,
    // Reserved for Phase 2: Compute Kernels
    pub compute_kernels: Vec<String>, 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetEntry {
    pub id: String,
    pub kind: AssetKind,
    /// Offset relative to the start of the DECOMPRESSED payload
    pub offset: u64,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AssetKind {
    SumiSource, // The user's S2L/WGSL code
    TexturePng,
    MeshGltf,   // Reserved for Phase 3
}

impl GyoshoFile {
    /// Encodes a project into the .gyo binary format with ZSTD compression
    pub fn write_new<W: Write + std::io::Seek>(
        writer: &mut W,
        manifest: &Manifest,
        raw_payload: &[u8]
    ) -> anyhow::Result<()> {
        // 1. Serialize Manifest
        let manifest_bytes = bincode::serialize(manifest)?;
        
        // 2. Compress Payload (Level 3 is a good balance of speed/ratio)
        let mut encoder = zstd::Encoder::new(Vec::new(), 3)?; 
        encoder.write_all(raw_payload)?;
        let compressed_payload = encoder.finish()?;

        // 3. Construct and Write File
        let file = GyoshoFile {
            magic: *MAGIC,
            version: 1,
            manifest_len: manifest_bytes.len() as u32,
            manifest_bytes,
            compressed_payload,
        };

        file.write(writer)?;
        Ok(())
    }

    /// Decodes the Manifest purely to peek at metadata (Fast)
    pub fn read_manifest<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R
    ) -> anyhow::Result<Manifest> {
        let file = GyoshoFile::read(reader)?;
        let manifest: Manifest = bincode::deserialize(&file.manifest_bytes)?;
        Ok(manifest)
    }
}
