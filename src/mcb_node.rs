use crate::*;

pub enum CommandType {
    Read,
    Write,
    StateChange,
}
pub struct Request {
    pub address: u16,
    pub command: CommandType,
    _data_value: [u16; MAX_FRAME_SIZE],
}
pub struct Node<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    _state: STATE,
    interface: INTERFACE,
}

/// These functions may be used on any Mcb struct
impl<STAT, INTF> Node<STAT, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn init(self) -> Node<Config, INTF> {
        Node {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Node<Config, INTF>
where
    INTF: PhysicalInterface,
{
    fn write_internal(&mut self, add: u16, cmd: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[HEADER_IDX] = self.frame.address;
        self.frame.raw[COMMAND_IDX] = cmd + (add << 4);
        self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);

        let built_frame = &self.frame.raw[..7];

        self.interface.raw_write(built_frame)
    }

    pub fn error(&mut self, addcmd: u16, err: u32) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = err as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (err >> 16) as u16;

        self.write_internal(0, CFG_ERR_BIT | addcmd)
    }

    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn read(&mut self) -> Result<Request, IntfError> {
        let data = match self.interface.raw_read() {
            Ok(IntfResult::Data(value)) => value,
            _ => return Err(IntfError::Interface),
        };

        if data[6] != self.interface.crc_checksum(&data[..6]) {
            return Err(IntfError::Crc);
        }

        let command = match data[1] & 0xEu16 {
            CFG_STD_READ => CommandType::Read,
            CFG_STD_WRITE => CommandType::Write,
            _ => return Err(IntfError::Interface),
        };

        Ok(Request {
            address: data[COMMAND_IDX] >> 4,
            command,
            _data_value: *data,
        })
    }

    pub fn listen(&self) -> Result<IntfResult, IntfError> {
        self.interface.is_data2read()
    }

    pub fn into_cyclic(self) -> Node<Cyclic, INTF> {
        Node {
            frame: self.frame,
            _state: Cyclic,
            interface: self.interface,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Node<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    fn write_internal(&mut self, add: u16, cmd: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[HEADER_IDX] = self.frame.address;
        self.frame.raw[COMMAND_IDX] = cmd + (add << 4);
        self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);

        let built_frame = &self.frame.raw[..7];

        self.interface.raw_write(built_frame)
    }

    pub fn writeu8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn read(&mut self) -> Result<Request, IntfError> {
        let data = match self.interface.raw_read() {
            Ok(IntfResult::Data(value)) => value,
            _ => return Err(IntfError::Interface),
        };

        let command = match data[1] & 0xEu16 {
            CFG_STD_READ => CommandType::Read,
            CFG_STD_WRITE => CommandType::Write,
            _ => return Err(IntfError::Interface),
        };

        Ok(Request {
            address: data[COMMAND_IDX] >> 4,
            command,
            _data_value: *data,
        })
    }

    pub fn into_config(self) -> Node<Config, INTF> {
        Node {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
        }
    }
}

pub fn create_node_mcb<INTF: PhysicalInterface>(interface: Option<INTF>) -> Node<Init, INTF> {
    let interface_in = interface.unwrap();
    Node {
        frame: Frame {
            address: 0u16,
            raw: [0u16; MAX_FRAME_SIZE],
        },
        _state: Init,
        interface: interface_in,
    }
}
