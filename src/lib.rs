#![crate_type = "lib"]
#![crate_name = "mcb"]

const CFG_EXT_BIT: u16 = 0x0001;
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

pub enum IntfResult {
    Success,
    Ready,
}

#[derive(Debug)]
pub enum IntfError {
    Interface,
    Access,
}

#[derive(Clone, Copy)]
pub struct Frame {
    address: u16,
    command: u16,
    raw: [u16; 7],
    crc: u16,
}
pub trait PhysicalInterface {
    fn initialize(&self) -> Result<IntfResult, IntfError> {
        panic!("Implementation of physical interface is required");
    }
    fn raw_write(&self, frame: &Frame) -> Result<IntfResult, IntfError> {
        panic!("Implementation of physical interface is required");
    }
    fn raw_read(&self, frame: &Frame) -> Result<IntfResult, IntfError> {
        panic!("Implementation of physical interface is required");
    }
}

/// Mcb State interface
pub struct Master<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    state: STATE,
    interface: INTERFACE,
}

pub struct Slave<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    state: STATE,
    interface: INTERFACE,
}

// Type States for MODE in State
pub struct Init;
pub struct Config;
pub struct Cyclic;

/// These functions may be used on any Mcb struct
impl<INTF> Master<Init, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn init(self) -> Master<Config, INTF> {
        Master {
            frame: self.frame,
            state: Config,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Master<Config, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[0] = self.frame.address;
        self.frame.raw[1] = CFG_STD_WRITE + (add << 4);
        self.frame.raw[2] = data as u16;
        self.frame.raw[6] = 0u16;

        self.interface.raw_write(&self.frame)
    }

    pub fn readu8(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[0] = self.frame.address;
        self.frame.raw[1] = CFG_STD_READ + (add << 4);
        self.frame.raw[6] = 0u16;

        return self.interface.raw_read(&self.frame);
    }

    pub fn into_cyclic(self) -> Master<Cyclic, INTF> {
        Master {
            frame: self.frame,
            state: Cyclic,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Master<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn write(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        return self.interface.raw_write(&self.frame);
    }

    pub fn read(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        return self.interface.raw_read(&self.frame);
    }

    pub fn into_config(self) -> Master<Config, INTF> {
        Master {
            frame: self.frame,
            state: Config,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb struct
impl<STAT, INTF> Slave<STAT, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn init(self) -> Slave<Config, INTF> {
        Slave {
            frame: self.frame,
            state: Config,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Slave<Config, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[0] = self.frame.address;
        self.frame.raw[1] = CFG_STD_ACK + (add << 4);
        self.frame.raw[2] = data as u16;
        self.frame.raw[6] = 0u16;

        return self.interface.raw_write(&self.frame);
    }

    pub fn readu8(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[0] = self.frame.address;
        self.frame.raw[1] = CFG_IDLE;
        self.frame.raw[6] = 0u16;

        return self.interface.raw_read(&self.frame);
    }

    pub fn listen(&self) -> Result<IntfResult, IntfError> {
        Ok(IntfResult::Success)
    }

    pub fn into_cyclic(self) -> Slave<Cyclic, INTF> {
        Slave {
            frame: self.frame,
            state: Cyclic,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Slave<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn write(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        return self.interface.raw_write(&self.frame);
    }

    pub fn read(self, add: u16) -> Result<IntfResult, IntfError> {
        return self.interface.raw_read(&self.frame);
    }

    pub fn into_config(self) -> Slave<Config, INTF> {
        Slave {
            frame: self.frame,
            state: Config,
            interface: self.interface,
        }
    }
}

pub fn create_master_mcb<INTF: PhysicalInterface>(interface: Option<INTF>) -> Master<Init, INTF> {
    let interface_in = interface.unwrap();
    match interface_in.initialize() {
        Err(e) => panic!("Interface is not initialized because of error: {:?}", e),
        _ => (),
    }
    Master {
        frame: Frame {
            address: 0u16,
            command: 0u16,
            raw: [0u16; 7],
            crc: 0u16,
        },
        state: Init,
        interface: interface_in,
    }
}

pub fn create_slave_mcb<INTF: PhysicalInterface>(interface: Option<INTF>) -> Slave<Init, INTF> {
    let interface_in = interface.unwrap();
    match interface_in.initialize() {
        Err(e) => panic!("Interface is not initialized because of error: {:?}", e),
        _ => (),
    }
    Slave {
        frame: Frame {
            address: 0u16,
            command: 0u16,
            raw: [0u16; 7],
            crc: 0u16,
        },
        state: Init,
        interface: interface_in,
    }
}
