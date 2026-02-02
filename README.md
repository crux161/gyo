# gyo

**gyo** is the reference implementation and specification for the `.gyo` file format—the universal export container for Gyosho shaders.

This repository serves as the "Rigid Spine" of the ecosystem, providing the strict schema, binary serialization logic, and a reference runtime (`hanga`) to verify that exported files can be correctly loaded and rendered.

---

## 📦 The `.gyo` Format

The primary purpose of this workspace is to define and maintain the `.gyo` binary standard.

- **Purpose:** To serve as a highly portable, compressed container for Gyosho shader code and assets.
- **Structure:**
    1.  **Header:** Magic bytes (`GYO1`) + Versioning.
    2.  **Manifest:** A Bincode-serialized metadata block describing the contained assets (Shaders, Textures, Compute Kernels).
    3.  **Payload:** A single Zstd-compressed blob containing the raw data.

---

## 🏗️ Workspace Structure

- **`crates/gyo_core`**
    - The **Specification**.
    - Contains the `GyoshoFile`, `Manifest`, and `AssetEntry` definitions.
    - Implements the canonical `read` and `write` methods for the format.

- **`crates/hanga`** ("Print")
    - The **Reference Runtime**.
    - A minimal WGPU renderer designed solely to verify that `.gyo` files can be loaded, decompressed, and executed on the GPU.
    - Includes the `ProjectLoader` for parsing the binary format at runtime.

- **`crates/hanga_traits`**
    - Defines the `Runtime` trait contract for applications that wish to consume `.gyo` files.

---

## 🌊 Verification: The Hokusai Benchmark

The `hokusai` example is the compliance test for the `.gyo` format. It verifies the full round-trip pipeline:

1.  **Generate:** Creates a valid `.gyo` file in memory from raw shader strings.
2.  **Serialize:** Writes the binary format (Manifest + Compressed Payload) to a buffer.
3.  **Deserialize:** Reads the buffer back using the `ProjectLoader`.
4.  **Execute:** Renders the loaded shader to the screen to prove data integrity.

### Running the Test

```bash
cargo run -p hanga --example hokusai
```

### 🛠️ Technology Stack
- Serialization: bincode (Metadata)

- Compression: zstd (Payload)

- Runtime: wgpu (v22.1) + winit (v0.30)

### 📅 Development Status
## Phase 1: Format Definition (Complete)

[x] Binary Format Spec (GYO1)

[x] Core Serialization Library (gyo_core)

[x] Reference Loader & Decompressor

[x] Verification Runtime (hanga)

## Next Steps

[ ] Phase 2: Compute Shader Support in .gyo schema

## License: MIT
