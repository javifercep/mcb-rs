use crate::*;

/// Mcb State interface
pub struct Main<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    _state: STATE,
    interface: INTERFACE,
}

/// These functions may be used on any Mcb struct
impl<INTF> Main<Init, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn init(self) -> Main<Config, INTF> {
        Main {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Main<Config, INTF>
where
    INTF: PhysicalInterface,
{
    fn internal_access(&mut self, add: u16, cmd: u16) -> Result<IntfResult, IntfError> {
        if add > MAX_ADDRESS {
            return Err(IntfError::AddressOutOfIndex);
        }

        self.frame.raw[HEADER_IDX] = self.frame.address;
        self.frame.raw[COMMAND_IDX] = cmd + (add << 4);
        self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw[..6]);

        let built_frame = &self.frame.raw[..7];

        match self.interface.raw_write(built_frame) {
            Ok(IntfResult::Success) => (),
            _ => return Err(IntfError::Interface),
        }

        let mut is_ready = self.interface.is_data2read();

        while let Ok(IntfResult::Empty) = is_ready {
            is_ready = self.interface.is_data2read();
        }

        let data = match self.interface.raw_read() {
            Ok(IntfResult::Data(value)) => value,
            _ => return Err(IntfError::Interface),
        };

        if data[6] != self.interface.crc_checksum(&data[..6]) {
            return Err(IntfError::Crc);
        }

        if data[1] != CFG_STD_ACK + (add << 4) {
            return Err(IntfError::Access(
                (data[2] as u32) | ((data[3] as u32) << 16),
            ));
        }

        Ok(IntfResult::Data(data))
    }

    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        match self.internal_access(add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn writeu32(&mut self, add: u16, data: u32) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (data >> 16) as u16;

        match self.internal_access(add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn readu8(&mut self, add: u16) -> Result<u8, IntfError> {
        match self.internal_access(add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as u8),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn into_cyclic(self) -> Main<Cyclic, INTF> {
        Main {
            frame: self.frame,
            _state: Cyclic,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Main<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    fn internal_access(&mut self, add: u16, cmd: u16) -> Result<IntfResult, IntfError> {
        if add > MAX_ADDRESS {
            return Err(IntfError::AddressOutOfIndex);
        }

        self.frame.raw[HEADER_IDX] = self.frame.address;
        self.frame.raw[COMMAND_IDX] = cmd + (add << 4);
        self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);

        let built_frame = &self.frame.raw[..7];

        match self.interface.raw_write(built_frame) {
            Ok(IntfResult::Success) => (),
            _ => return Err(IntfError::Interface),
        }

        let mut is_ready = self.interface.is_data2read();

        while let Ok(IntfResult::Empty) = is_ready {
            is_ready = self.interface.is_data2read();
        }

        let data = match self.interface.raw_read() {
            Ok(IntfResult::Data(value)) => value,
            _ => return Err(IntfError::Interface),
        };

        if data[1] != CFG_STD_ACK + (add << 4) {
            return Err(IntfError::Access(
                (data[2] as u32) | ((data[3] as u32) << 16),
            ));
        }

        Ok(IntfResult::Data(data))
    }

    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        match self.internal_access(add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn readu8(&mut self, add: u16) -> Result<u8, IntfError> {
        match self.internal_access(add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as u8),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn into_config(self) -> Main<Config, INTF> {
        Main {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
        }
    }
}

pub fn create_main_mcb<INTF: PhysicalInterface>(interface: Option<INTF>) -> Main<Init, INTF> {
    let interface_in = interface.unwrap();
    Main {
        frame: Frame {
            address: 0u16,
            raw: [0u16; MAX_FRAME_SIZE],
        },
        _state: Init,
        interface: interface_in,
    }
}
