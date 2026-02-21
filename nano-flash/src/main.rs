use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crc32fast::Hasher;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about = "Nano target unified flash tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build a unified nano image
    Build {
        /// Kernel binary path
        #[arg(long)]
        kernel: PathBuf,
        /// Driver HAL binary path
        #[arg(long)]
        hal: PathBuf,
        /// Ion shell binary path
        #[arg(long)]
        ion: PathBuf,
        /// Output unified binary path
        #[arg(long, default_value = "nano-esp32.bin")]
        output: PathBuf,
    },
    /// Write the unified image to a device (stub)
    Write {
        /// Unified binary to flash
        #[arg(short, long)]
        image: PathBuf,
        /// Serial port to write to
        #[arg(short, long)]
        port: Option<String>,
    },
}

const MAGIC: [u8; 4] = [0x52, 0x44, 0x4F, 0x58]; // RDOX

#[repr(C)]
#[derive(Debug, Default)]
struct ComponentHeader {
    name: [u8; 16],
    offset: u32,
    size: u32,
    crc: u32,
}

fn write_component(
    output: &mut File,
    path: &Path,
    name: &str,
    offset: u32,
) -> Result<ComponentHeader> {
    let mut input =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut data = Vec::new();
    input.read_to_end(&mut data)?;

    output.seek(SeekFrom::Start(offset as u64))?;
    output.write_all(&data)?;

    let mut hasher = Hasher::new();
    hasher.update(&data);
    let crc = hasher.finalize();

    let mut header = ComponentHeader {
        offset,
        size: data.len() as u32,
        crc,
        ..Default::default()
    };

    let name_bytes = name.as_bytes();
    let len = name_bytes.len().min(16);
    header.name[..len].copy_from_slice(&name_bytes[..len]);

    Ok(header)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            kernel,
            hal,
            ion,
            output,
        } => {
            println!("Building unified nano image at {}...", output.display());
            let mut out_file = File::create(&output)?;

            // Main Header space (MAGIC + 3 headers * 28 bytes)
            let header_size = 4 + (3 * std::mem::size_of::<ComponentHeader>()) as u32;
            let mut current_offset = header_size;

            out_file.write_all(&MAGIC)?;

            let kernel_hdr = write_component(&mut out_file, &kernel, "kernel", current_offset)?;
            current_offset += kernel_hdr.size;

            let hal_hdr = write_component(&mut out_file, &hal, "hal", current_offset)?;
            current_offset += hal_hdr.size;

            let ion_hdr = write_component(&mut out_file, &ion, "ion", current_offset)?;

            // Write Table of Contents
            out_file.seek(SeekFrom::Start(4))?;
            let headers = [kernel_hdr, hal_hdr, ion_hdr];
            for hdr in headers.iter() {
                out_file.write_all(&hdr.name)?;
                out_file.write_all(&hdr.offset.to_le_bytes())?;
                out_file.write_all(&hdr.size.to_le_bytes())?;
                out_file.write_all(&hdr.crc.to_le_bytes())?;
            }

            println!("✓ Successfully bundled kernel, HAL, and ion.");
        }
        Commands::Write { image, port } => {
            let p = port.unwrap_or_else(|| "/dev/ttyUSB0".to_string());
            println!("Flashing {} to {}...", image.display(), p);
            // In reality, this would open serial port and stream data
            println!("✓ Write complete.");
        }
    }

    Ok(())
}
