use crate::*;
#[derive(Debug)]
pub enum CommandType {
    Read,
    Write,
    ExtRead,
    ExtWrite,
    StateChange,
}
pub struct Request {
    pub subnode: u8,
    pub address: u16,
    pub command: CommandType,
    data_value: [u16; MAX_FRAME_SIZE],
}
pub struct Node<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    _state: STATE,
    interface: INTERFACE,
    ext_mode: ExtMode,
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
            ext_mode: self.ext_mode,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Node<Config, INTF>
where
    INTF: PhysicalInterface,
{
    fn write_internal(&mut self, add: u16, cmd: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[HEADER_IDX] = self.frame.subnode as u16;
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

    pub fn ack(&mut self, add: u16) -> Result<IntfResult, IntfError> {
        self.write_internal(add, CFG_STD_ACK)
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

    pub fn write_str(&mut self, add: u16, data: &str) -> Result<IntfResult, IntfError> {
        let size = data.len();
        if size < MAX_STD_CFG_DATA {
            let mut char_list = data.chars();
            let mut char_value = char_list.next();
            let mut count = CFG_DATA_IDX;
            while char_value.is_some() {
                self.frame.raw[count] = char_value.unwrap() as u16;
                char_value = char_list.next();
                if char_value.is_some() {
                    self.frame.raw[count] |= (char_value.unwrap() as u16) << 8;
                    char_value = char_list.next();
                } else {
                    self.frame.raw[count] &= 0x0fu16;
                    break;
                }
                count += 1;
            }
            self.frame.raw[count] = 0u16;
            self.write_internal(add, CFG_STD_ACK)
        } else {
            self.frame.raw[HEADER_IDX] = self.frame.subnode as u16;

            match self.ext_mode {
                ExtMode::Segmented => {
                    let mut char_list = data.chars();
                    let mut char_value = char_list.next();
                    let mut count = CFG_DATA_IDX;

                    while char_value.is_some() {
                        self.frame.raw[count] = char_value.unwrap() as u16;
                        char_value = char_list.next();
                        if char_value.is_some() {
                            self.frame.raw[count] |= (char_value.unwrap() as u16) << 8;
                            char_value = char_list.next();
                        } else {
                            self.frame.raw[count] &= 0x0fu16;
                            count += 1;
                            break;
                        }

                        if count == 5 {
                            self.frame.raw[COMMAND_IDX] = CFG_EXT_ACK + (add << 4);
                            self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);
                            let built_frame = &self.frame.raw[..7];
                            if self.interface.raw_write(built_frame).is_err() {
                                return Err(IntfError::Interface);
                            }
                            count = CFG_DATA_IDX;
                        } else {
                            count += 1;
                        }
                    }
                    self.frame.raw[COMMAND_IDX] = CFG_STD_ACK + (add << 4);
                    self.frame.raw[count] = 0u16;
                    self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);
                    let built_frame = &self.frame.raw[..7];
                    self.interface.raw_write(built_frame)
                }
                ExtMode::Extended => {
                    self.frame.raw[COMMAND_IDX] = CFG_EXT_ACK + (add << 4);
                    self.frame.raw[CFG_DATA_IDX] = size as u16;
                    self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);

                    let mut char_list = data.chars();
                    let mut char_value = char_list.next();
                    let mut count = 7;
                    while char_value.is_some() {
                        self.frame.raw[count] = char_value.unwrap() as u16;
                        char_value = char_list.next();
                        if char_value.is_some() {
                            self.frame.raw[count] |= (char_value.unwrap() as u16) << 8;
                            char_value = char_list.next();
                        } else {
                            self.frame.raw[count] &= 0x0fu16;
                            break;
                        }
                        count += 1;
                    }
                    self.frame.raw[count] = 0u16;
                    let built_frame = &self.frame.raw[..7 + size];

                    self.interface.raw_write(built_frame)
                }
            }
        }
    }

    pub fn read(&mut self) -> Result<Request, IntfError> {
        let mut data = match self.interface.raw_read() {
            Ok(IntfResult::Data(value)) => value,
            _ => return Err(IntfError::Interface),
        };

        if data[6] != self.interface.crc_checksum(&data[..6]) {
            return Err(IntfError::Crc);
        }

        let command = match data[1] & 0xfu16 {
            CFG_STD_READ => CommandType::Read,
            CFG_STD_WRITE => CommandType::Write,
            CFG_EXT_READ => CommandType::ExtRead,
            CFG_EXT_WRITE => CommandType::ExtWrite,
            _ => return Err(IntfError::WrongCommand),
        };

        if let ExtMode::Segmented = self.ext_mode {
            let mut count = 6;
            loop {
                if self.ack(data[COMMAND_IDX] >> 4).is_err() {
                    return Err(IntfError::Interface);
                }

                let mut is_ready = self.listen();

                while let Ok(IntfResult::Empty) = is_ready {
                    is_ready = self.listen();
                }

                let data_segment = match self.interface.raw_read() {
                    Ok(IntfResult::Data(value)) => value,
                    _ => return Err(IntfError::Interface),
                };

                if data_segment[6] != self.interface.crc_checksum(&data[..6]) {
                    return Err(IntfError::Crc);
                }

                data[count..count + 4].copy_from_slice(&data_segment[2..6]);

                count += 4;

                if (data_segment[1] & 0x1u16) != CFG_EXT_BIT {
                    break;
                }
            }
        }

        Ok(Request {
            subnode: data[HEADER_IDX] as u8 & 0xfu8,
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

    pub fn listen(&mut self) -> Result<IntfResult, IntfError> {
        self.interface.is_data2read()
    }

    pub fn into_cyclic(self) -> Node<Cyclic, INTF> {
        Node {
            frame: self.frame,
            _state: Cyclic,
            interface: self.interface,
            ext_mode: self.ext_mode,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Node<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn into_config(self) -> Node<Config, INTF> {
        Node {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
            ext_mode: self.ext_mode,
        }
    }
}

pub fn create_node_mcb<INTF: PhysicalInterface>(
    interface: Option<INTF>,
    mode: ExtMode,
    subnode: u8,
) -> Node<Init, INTF> {
    let interface_in = interface.unwrap();
    Node {
        frame: Frame {
            _address: 0u16,
            subnode,
            raw: [0u16; MAX_FRAME_SIZE],
        },
        _state: Init,
        interface: interface_in,
        ext_mode: mode,
    }
}
