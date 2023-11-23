#![crate_type = "lib"]

#![crate_name = "mcb_rust"]

const CFG_EXT_BIT:  u16 = 0x0001;
const CFG_STD_READ: u16 = 0x0002;
const CFG_EXT_READ: u16 = CFG_STD_READ + CFG_EXT_BIT; /// 0x0003
const CFG_STD_WRITE: u16 = 0x0004;
const CFG_EXT_WRITE: u16 = CFG_EXT_READ + CFG_EXT_BIT; /// 0x0005
const CFG_STD_ACK:   u16 = 0x0006;
const CFG_EXT_ACK:   u16 = CFG_STD_ACK + CFG_EXT_BIT; /// 0x0007
const CFG_IDLE:      u16 = 0x000E;

#[derive(Debug)]
pub enum OperationResult{
    Success,
    Fail,
}

pub struct McbData{
    pub data: u64,
}
pub trait PhysicalInterface {
    fn raw_write(&self) -> OperationResult {return OperationResult::Success}
    fn raw_read(&self) -> OperationResult {return OperationResult::Success}
}

impl PhysicalInterface for McbData {}

/// Initialize the library.
///
/// This function is required in systems where device peripherals requires
/// a specific configuration. For example, when the MCB protocol is 
/// implemented over SPI interfaces. A false result of this function
/// must be considered as critical error
/// 
/// ```rust
/// if mcb_rust::init() == false {
///     panic!("Library cannot be initialized");
/// }
/// ```
pub fn init() -> bool {
    return true;
}

pub fn write(add: u16, data: McbData) -> OperationResult {
    let mut raw_frame: [u16; 6] = [0; 6];
    let test: [u16; 4];

    raw_frame[0] = CFG_STD_WRITE + (add << 4);
    test = bytemuck::cast(data.data);
    raw_frame[0..4].copy_from_slice(&test);
    raw_frame[5] = 0u16;

    return data.raw_write();
}

pub fn read(add: u16, data: McbData) -> OperationResult {
    let mut raw_frame: [u16; 6] = [0; 6];
    let test: [u16; 4];

    raw_frame[0] = CFG_STD_READ + (add << 4);
    test = bytemuck::cast(data.data);
    raw_frame[0..4].copy_from_slice(&test);
    raw_frame[5] = 0u16;

    return data.raw_read();
}

pub fn enable_cyclic() -> OperationResult {
    return OperationResult::Success;
}

pub fn disable_cyclic() -> OperationResult {
    return OperationResult::Success;
}
