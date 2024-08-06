use crate::*;

/// Mcb State interface
pub struct Main<STATE, INTERFACE: PhysicalInterface> {
    frame: Frame,
    _state: STATE,
    interface: INTERFACE,
    ext_mode: ExtMode,
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
            ext_mode: self.ext_mode,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Main<Config, INTF>
where
    INTF: PhysicalInterface,
{
    fn internal_access(
        &mut self,
        subnode: u8,
        add: u16,
        cmd: u16,
    ) -> Result<IntfResult, IntfError> {
        if add > MAX_ADDRESS {
            return Err(IntfError::AddressOutOfIndex);
        }

        self.frame.raw[HEADER_IDX] = subnode as u16;
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

        if (data[0] & 0xfu16) != subnode as u16 {
            return Err(IntfError::Access(0u32));
        }

        if (data[1] & 0xfffeu16) != (CFG_STD_ACK + (add << 4)) {
            return Err(IntfError::Access(
                (data[2] as u32) | ((data[3] as u32) << 16),
            ));
        }

        Ok(IntfResult::Data(data))
    }

    pub fn write_u8(&mut self, subnode: u8, add: u16, data: u8) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;

        match self.internal_access(subnode, add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn read_u8(&mut self, subnode: u8, add: u16) -> Result<u8, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as u8),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_i8(&mut self, subnode: u8, add: u16, data: i8) -> Result<IntfResult, IntfError> {
        self.write_u8(subnode, add, data as u8)
    }

    pub fn read_i8(&mut self, subnode: u8, add: u16) -> Result<i8, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as i8),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_u16(&mut self, subnode: u8, add: u16, data: u16) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data;

        match self.internal_access(subnode, add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn read_u16(&mut self, subnode: u8, add: u16) -> Result<u16, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX]),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_i16(&mut self, subnode: u8, add: u16, data: i16) -> Result<IntfResult, IntfError> {
        self.write_u16(subnode, add, data as u16)
    }

    pub fn read_i16(&mut self, subnode: u8, add: u16) -> Result<i16, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as i16),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_u32(&mut self, subnode: u8, add: u16, data: u32) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (data >> 16) as u16;

        match self.internal_access(subnode, add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn read_u32(&mut self, subnode: u8, add: u16) -> Result<u32, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => {
                Ok(value[CFG_DATA_IDX] as u32 | ((value[CFG_DATA_IDX + 1] as u32) << 16))
            }
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_i32(&mut self, subnode: u8, add: u16, data: i32) -> Result<IntfResult, IntfError> {
        self.write_u32(subnode, add, data as u32)
    }

    pub fn read_i32(&mut self, subnode: u8, add: u16) -> Result<i32, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => {
                Ok(value[CFG_DATA_IDX] as i32 | ((value[CFG_DATA_IDX + 1] as i32) << 16))
            }
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_u64(&mut self, subnode: u8, add: u16, data: u64) -> Result<IntfResult, IntfError> {
        self.frame.raw[CFG_DATA_IDX] = data as u16;
        self.frame.raw[CFG_DATA_IDX + 1] = (data >> 16) as u16;
        self.frame.raw[CFG_DATA_IDX + 2] = (data >> 32) as u16;
        self.frame.raw[CFG_DATA_IDX + 3] = (data >> 48) as u16;

        match self.internal_access(subnode, add, CFG_STD_WRITE) {
            Ok(_) => Ok(IntfResult::Success),
            Err(e) => Err(e),
        }
    }

    pub fn read_u64(&mut self, subnode: u8, add: u16) -> Result<u64, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as u64
                | ((value[CFG_DATA_IDX + 1] as u64) << 16)
                | ((value[CFG_DATA_IDX + 2] as u64) << 32)
                | ((value[CFG_DATA_IDX + 3] as u64) << 48)),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_i64(&mut self, subnode: u8, add: u16, data: i64) -> Result<IntfResult, IntfError> {
        self.write_u64(subnode, add, data as u64)
    }

    pub fn read_i64(&mut self, subnode: u8, add: u16) -> Result<i64, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok(value[CFG_DATA_IDX] as i64
                | ((value[CFG_DATA_IDX + 1] as i64) << 16)
                | ((value[CFG_DATA_IDX + 2] as i64) << 32)
                | ((value[CFG_DATA_IDX + 3] as i64) << 48)),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_f32(&mut self, subnode: u8, add: u16, data: f32) -> Result<IntfResult, IntfError> {
        self.write_u32(subnode, add, data as u32)
    }

    pub fn read_f32(&mut self, subnode: u8, add: u16) -> Result<f32, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => {
                Ok((value[CFG_DATA_IDX] as u32 | ((value[CFG_DATA_IDX + 1] as u32) << 16)) as f32)
            }
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_f64(&mut self, subnode: u8, add: u16, data: f64) -> Result<IntfResult, IntfError> {
        self.write_u64(subnode, add, data as u64)
    }

    pub fn read_f64(&mut self, subnode: u8, add: u16) -> Result<f64, IntfError> {
        match self.internal_access(subnode, add, CFG_STD_READ) {
            Ok(IntfResult::Data(value)) => Ok((value[CFG_DATA_IDX] as u64
                | ((value[CFG_DATA_IDX + 1] as u64) << 16)
                | ((value[CFG_DATA_IDX + 2] as u64) << 32)
                | ((value[CFG_DATA_IDX + 3] as u64) << 48))
                as f64),
            Err(e) => Err(e),
            _ => Err(IntfError::Interface),
        }
    }

    pub fn write_str(
        &mut self,
        subnode: u8,
        add: u16,
        data: &str,
    ) -> Result<IntfResult, IntfError> {
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
                    self.frame.raw[count] &= 0xffu16;
                }
                count += 1;
            }

            self.frame.raw[count] = 0u16;
            match self.internal_access(subnode, add, CFG_STD_WRITE) {
                Ok(_) => Ok(IntfResult::Success),
                Err(e) => Err(e),
            }
        } else {
            self.frame.raw[HEADER_IDX] = subnode as u16;

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
                            self.frame.raw[count] &= 0xffu16;
                            count += 1;
                            break;
                        }

                        if count == 5 {
                            self.frame.raw[COMMAND_IDX] = CFG_EXT_WRITE + (add << 4);
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
                    self.frame.raw[COMMAND_IDX] = CFG_STD_WRITE + (add << 4);
                    self.frame.raw[count] = 0u16;
                    self.frame.raw[6] = self.interface.crc_checksum(&self.frame.raw);
                    let built_frame = &self.frame.raw[..7];
                    match self.interface.raw_write(built_frame) {
                        Ok(_) => Ok(IntfResult::Success),
                        Err(e) => Err(e),
                    }
                }
                ExtMode::Extended => {
                    self.frame.raw[COMMAND_IDX] = CFG_EXT_WRITE + (add << 4);
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
                            self.frame.raw[count] &= 0xffu16;
                            break;
                        }
                        count += 1;
                    }
                    self.frame.raw[count] = 0u16;
                    let built_frame = &self.frame.raw[..7 + size];

                    match self.interface.raw_write(built_frame) {
                        Ok(_) => Ok(IntfResult::Success),
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }

    pub fn read_str(&mut self, subnode: u8, add: u16) -> Result<String, IntfError> {
        match self.ext_mode {
            ExtMode::Extended => {
                let data_words = match self.internal_access(subnode, add, CFG_STD_READ) {
                    Ok(IntfResult::Data(value)) => value,
                    _ => return Err(IntfError::Interface),
                };

                let data_bytes = unsafe { data_words[2..].align_to::<u8>().1 };

                if (data_words[1] & 0x1u16) != CFG_EXT_BIT {
                    let string_result: String = data_bytes[..]
                        .iter()
                        .take_while(|&&u| u != 0)
                        .map(|&u| std::char::from_u32(u as u32).unwrap())
                        .collect();

                    Ok(string_result)
                } else {
                    let string_result: String = data_bytes[10..]
                        .iter()
                        .take_while(|&&u| u != 0)
                        .map(|&u| std::char::from_u32(u as u32).unwrap())
                        .collect();

                    Ok(string_result)
                }
            }
            ExtMode::Segmented => {
                let mut result = String::new();
                loop {
                    let data_words = match self.internal_access(subnode, add, CFG_STD_READ) {
                        Ok(IntfResult::Data(value)) => value,
                        _ => return Err(IntfError::Interface),
                    };

                    let data_bytes = unsafe { data_words[2..6].align_to::<u8>().1 };
                    let string_segment: String = data_bytes[..]
                        .iter()
                        .take_while(|&&u| u != 0)
                        .map(|&u| std::char::from_u32(u as u32).unwrap())
                        .collect();

                    result.push_str(&string_segment);

                    if (data_words[1] & 0x1u16) != CFG_EXT_BIT {
                        break;
                    }
                }
                Ok(result)
            }
        }
    }

    pub fn into_cyclic(self) -> Main<Cyclic, INTF> {
        Main {
            frame: self.frame,
            _state: Cyclic,
            interface: self.interface,
            ext_mode: self.ext_mode,
        }
    }
}

/// These functions may be used on any Mcb in config State
impl<INTF> Main<Cyclic, INTF>
where
    INTF: PhysicalInterface,
{
    pub fn into_config(self) -> Main<Config, INTF> {
        Main {
            frame: self.frame,
            _state: Config,
            interface: self.interface,
            ext_mode: self.ext_mode,
        }
    }
}

pub fn create_main_mcb<INTF: PhysicalInterface>(
    interface: Option<INTF>,
    mode: ExtMode,
    subnode: u8,
) -> Main<Init, INTF> {
    let interface_in = interface.unwrap();
    Main {
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
