use crate::*;

pub enum CommandType {
    Read,
    Write,
    StateChange,
}
pub struct Request {
    pub address: u16,
    pub command: CommandType,
    data_value: [u16; MAX_FRAME_SIZE],
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

    pub fn write_u8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn write_i8(&mut self, add: u16, data: i8) -> Result<IntfResult, IntfError> {
        self.write_u8(add, data as u8)
    }

    pub fn write_u16(&mut self, add: u16, data: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn write_i16(&mut self, add: u16, data: i16) -> Result<IntfResult, IntfError> {
        self.write_u16(add, data as u16)
    }

    pub fn write_u32(&mut self, add: u16, data: u32) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (data >> 16) as u16;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn write_i32(&mut self, add: u16, data: i32) -> Result<IntfResult, IntfError> {
        self.write_u32(add, data as u32)
    }

    pub fn write_u64(&mut self, add: u16, data: u64) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (data >> 16) as u16;
        self.frame.raw[CFG_DATA_IDX + 2] = (data >> 32) as u16;
        self.frame.raw[CFG_DATA_IDX + 3] = (data >> 48) as u16;

        self.write_internal(add, CFG_STD_ACK)
    }

    pub fn write_i64(&mut self, add: u16, data: i64) -> Result<IntfResult, IntfError> {
        self.write_u64(add, data as u64)
    }

    pub fn write_f32(&mut self, add: u16, data: f32) -> Result<IntfResult, IntfError> {
        self.write_u32(add, data as u32)
    }

    pub fn write_f64(&mut self, add: u16, data: f64) -> Result<IntfResult, IntfError> {
        self.write_u64(add, data as u64)
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
            data_value: *data,
        })
    }

    pub fn get_data_u8(&self, request: &Request) -> u8 {
        request.data_value[CFG_DATA_IDX] as u8
    }

    pub fn get_data_i8(&self, request: &Request) -> i8 {
        self.get_data_u8(request) as i8
    }

    pub fn get_data_u16(&self, request: &Request) -> u16 {
        request.data_value[CFG_DATA_IDX]
    }

    pub fn get_data_i16(&self, request: &Request) -> i16 {
        self.get_data_u16(request) as i16
    }

    pub fn get_data_u32(&self, request: &Request) -> u32 {
        request.data_value[CFG_DATA_IDX] as u32
            | ((request.data_value[CFG_DATA_IDX + 1] as u32) << 16)
    }

    pub fn get_data_i32(&self, request: &Request) -> i32 {
        self.get_data_u32(request) as i32
    }

    pub fn get_data_u64(&self, request: &Request) -> u64 {
        request.data_value[CFG_DATA_IDX] as u64
            | ((request.data_value[CFG_DATA_IDX + 1] as u64) << 16)
            | ((request.data_value[CFG_DATA_IDX + 2] as u64) << 32)
            | ((request.data_value[CFG_DATA_IDX + 3] as u64) << 48)
    }

    pub fn get_data_i64(&self, request: &Request) -> i64 {
        self.get_data_u64(request) as i64
    }

    pub fn get_data_f32(&self, request: &Request) -> f32 {
        self.get_data_u32(request) as f32
    }

    pub fn get_data_f64(&self, request: &Request) -> f64 {
        self.get_data_u64(request) as f64
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

    pub fn write_u8(&mut self, add: u16, data: u8) -> Result<IntfResult, IntfError> {
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
            data_value: *data,
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
