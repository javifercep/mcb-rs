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
//!

pub mod mcb_main;
pub mod mcb_node;

const CFG_EXT_BIT: u16 = 0x0001;
const CFG_ERR_BIT: u16 = 0x0008;
const CFG_STD_READ: u16 = 0x0002;
const CFG_EXT_READ: u16 = CFG_STD_READ + CFG_EXT_BIT;
/// 0x0003
const CFG_STD_WRITE: u16 = 0x0004;
const CFG_EXT_WRITE: u16 = CFG_EXT_READ + CFG_EXT_BIT;
/// 0x0005
const CFG_STD_ACK: u16 = 0x0006;
const CFG_EXT_ACK: u16 = CFG_STD_ACK + CFG_EXT_BIT;
/// 0x0007
const CFG_IDLE: u16 = 0x000E;

pub const MAX_FRAME_SIZE: usize = 128;
pub const MAX_ADDRESS: u16 = 0x0FFF;

pub const HEADER_IDX: usize = 0;
pub const COMMAND_IDX: usize = 1;
pub const CFG_DATA_IDX: usize = 2;
pub const CYC_DATA_IDX: usize = 6;

#[derive(Debug)]
pub enum IntfResult {
    Success,
    Empty,
    Ready,
    Data([u16; MAX_FRAME_SIZE]),
}

#[derive(Debug)]
pub enum IntfError {
    Interface,
    Access(u32),
    AddressOutOfIndex,
}

#[derive(Clone, Copy)]
pub struct Frame {
    address: u16,
    command: u16,
    raw: [u16; MAX_FRAME_SIZE],
    crc: u16,
}
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

    fn is_data2read(&self) -> Result<IntfResult, IntfError>;

    fn crc_checksum(&self, frame: &[u16]) -> u16 {
        const XMODEM: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
        unsafe { XMODEM.checksum(&frame[1..5].align_to::<u8>().1) }
    }
}

/// Typestate Init
pub struct Init;
/// Typestate Config
pub struct Config;
/// Typestate Cyclic
pub struct Cyclic;
