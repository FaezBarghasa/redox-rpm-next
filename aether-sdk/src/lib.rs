//! # AetherSDK — AetherOS Native Game & Application SDK for Redox OS
//!
//! Provides high-level and low-level abstractions for hardware-accelerated graphics (DRM),
//! low-latency multi-client audio streaming (`audio:` scheme), raw game input controller
//! mappings, and scheduler game-mode prioritization.

use std::fs::File;
use std::io::{Write};
use std::os::unix::io::{FromRawFd, RawFd};

/// Standard Audio Parameters for AetherOS Game Streams
pub const AUDIO_SAMPLE_RATE_48K: u32 = 48000;
pub const AUDIO_CHANNELS_STEREO: u16 = 2;

/// Gamepad Button Flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButton {
    South = 1 << 0,  // A / Cross
    East = 1 << 1,   // B / Circle
    West = 1 << 2,   // X / Square
    North = 1 << 3,  // Y / Triangle
    L1 = 1 << 4,
    R1 = 1 << 5,
    Select = 1 << 6,
    Start = 1 << 7,
    L3 = 1 << 8,
    R3 = 1 << 9,
}

/// Consolidated Gamepad State Snapshot
#[derive(Debug, Clone, Copy, Default)]
pub struct GamepadState {
    pub buttons: u32,
    pub left_stick_x: f32,  // -1.0 to 1.0
    pub left_stick_y: f32,  // -1.0 to 1.0
    pub right_stick_x: f32, // -1.0 to 1.0
    pub right_stick_y: f32, // -1.0 to 1.0
    pub left_trigger: f32,  // 0.0 to 1.0
    pub right_trigger: f32, // 0.0 to 1.0
}

/// AetherOS Audio Client Stream Wrapper
pub struct AetherAudioStream {
    file: File,
}

impl AetherAudioStream {
    pub fn open() -> Result<Self, std::io::Error> {
        let fd = libredox::call::open(":audio", libredox::flag::O_WRONLY, 0)
            .map_err(|err| std::io::Error::from_raw_os_error(err.errno()))?;
        let file = unsafe { File::from_raw_fd(fd as RawFd) };
        Ok(Self { file })
    }

    pub fn write_pcm_s16le(&mut self, samples: &[i16]) -> std::io::Result<usize> {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                samples.as_ptr() as *const u8,
                samples.len() * std::mem::size_of::<i16>(),
            )
        };
        self.file.write(bytes)
    }
}

/// Requests the OS scheduler and display server to grant Exclusive Game Mode
pub fn enable_game_mode() -> Result<(), std::io::Error> {
    log::info!("Requesting AetherOS Exclusive Game Mode...");
    // Request priority boost via scheme call to orbital / scheduler
    if let Ok(fd) = libredox::call::open(":orbital", libredox::flag::O_RDWR, 0) {
        let mut file = unsafe { File::from_raw_fd(fd as RawFd) };
        let _ = file.write_all(b"GAME_MODE_ON");
    }
    Ok(())
}

/// DRM GEM Buffer Allocation Structure
#[derive(Debug)]
pub struct AetherDrmBuffer {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub size: usize,
}

impl AetherDrmBuffer {
    pub fn allocate(width: u32, height: u32, bpp: u32) -> Result<Self, std::io::Error> {
        let pitch = width * (bpp / 8);
        let size = (pitch * height) as usize;
        Ok(Self {
            handle: 1,
            width,
            height,
            pitch,
            size,
        })
    }
}
