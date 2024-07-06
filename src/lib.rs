#![crate_type = "lib"]
#![crate_name = "mcb"]

//! # mcb-rs
//!
//! `mcb-rs` implements the Motion Control Bus protocol:
//!   * [Motion Control Bus](https://drives.novantamotion.com/eve-core/mcb-overview)
//!
//! This library works over other protocols such as Ethernet,
//! SPI or USB. It is designed for embedded system where the
//! hardware abstraction needs to be created. This means that
//! this library requires the implementation of a struct implementing
//! the [`PhysicalInterface`] trait.
//!
//! # Examples
//!
//! ```
//! use mcb::mcb_main::{create_main_mcb, Main};
//! use mcb::{Config, Init, IntfError, IntfResult, PhysicalInterface, MAX_FRAME_SIZE};
//! use mcb::IntfResult::*;
//!
//! struct NewInterface;
//!
//! impl PhysicalInterface for NewInterface {
//!
//!     fn raw_write(&self, frame: &[u16]) -> Result<IntfResult, IntfError> {
//!         // your implementation
//!         Ok(Success)
//!     }
//!
//!     fn raw_read(&self) -> Result<IntfResult, IntfError> {
//!         // ignore this block. Created to pass cargo test --doc
//!         let mut msg = [0u16; MAX_FRAME_SIZE];
//!         // your implementation
//!         Ok(Data(Box::new(msg)))
//!     }
//!
//!     fn is_data2read(&self) -> Result<IntfResult, IntfError> {
//!         // your implementation
//!         Ok(Success)
//!     }
//! }
//! ```
//!
//! This crate contains both modules
//!  * Main
//!  * Node
//!
//! The main module is the expected to be used over SPI with the drives implementing these protocols:
//!  * [Capitan CORE](https://drives.novantamotion.com/cap-core/)
//!  * [Everest CORE](https://drives.novantamotion.com/eve-core/)
//!
//! The node module is expected to be used for bridge applications such as Turonet
//!  * [Hardware](https://github.com/javifercep/Turonet)
//!  * [Firmware](https://github.com/javifercep/turonet-rs)
//!

/// Module implementing Main devices
pub mod mcb_main;
/// Module implementing Node devices
pub mod mcb_node;

/// Maximum size of a single frame
pub const MAX_FRAME_SIZE: usize = 128;

const CFG_EXT_BIT: u16 = 0x0001;
const CFG_ERR_BIT: u16 = 0x0008;
const CFG_STD_READ: u16 = 0x0002;
const CFG_EXT_READ: u16 = CFG_STD_READ + CFG_EXT_BIT;
const CFG_STD_WRITE: u16 = 0x0004;
const CFG_EXT_WRITE: u16 = CFG_EXT_READ + CFG_EXT_BIT;
const CFG_STD_ACK: u16 = 0x0006;
const CFG_EXT_ACK: u16 = CFG_STD_ACK + CFG_EXT_BIT;
const CFG_IDLE: u16 = 0x000E;

const MAX_ADDRESS: u16 = 0x0FFF;

const HEADER_IDX: usize = 0;
const COMMAND_IDX: usize = 1;
const CFG_DATA_IDX: usize = 2;
const CYC_DATA_IDX: usize = 6;

/// Successful results of an MCB access
#[derive(Debug)]
pub enum IntfResult {
    Success,
    Empty,
    Ready,
    Data(Box<[u16; MAX_FRAME_SIZE]>),
}

/// Error results of an MCB access
#[derive(Debug)]
pub enum IntfError {
    Interface,
    Access(u32),
    AddressOutOfIndex,
    Crc,
}

#[derive(Clone, Copy)]
struct Frame {
    address: u16,
    raw: [u16; MAX_FRAME_SIZE],
}
/// This trait contains the implementation required to access to the Network/Bus
pub trait PhysicalInterface {
    /// This function is called everytime the procotol needs to access
    /// and write some data to the interface. The frame slice type is u16
    /// because this protocol works in words, specially when it is implemented
    /// over SPI
    fn raw_write(&self, frame: &[u16]) -> Result<IntfResult, IntfError>;

    /// This function is called everytime the procotol needs to access
    /// and read some data from the interface. The frame slice type is u16
    /// because this protocol works in words, specially when it is implemented
    /// over SPI
    fn raw_read(&self) -> Result<IntfResult, IntfError>;

    /// This function checks if there is data available to be read. If not needed, use
    /// the default implementation
    fn is_data2read(&self) -> Result<IntfResult, IntfError> {
        Ok(IntfResult::Success)
    }

    /// This trait is availabble to offer the option to compute the CRC through a HW
    /// accelerator or dedicated peripheral. Otherwise, the default implementation is
    /// available
    fn crc_checksum(&self, frame: &[u16]) -> u16 {
        const XMODEM: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
        unsafe { XMODEM.checksum(frame[..6].align_to::<u8>().1) }
    }
}

/// Typestate Init
pub struct Init;
/// Typestate Config
pub struct Config;
/// Typestate Cyclic
pub struct Cyclic;
