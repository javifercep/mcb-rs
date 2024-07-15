use mcb::{Config, Init, IntfError, IntfResult, PhysicalInterface, ExtMode, MAX_FRAME_SIZE};
use mcb::mcb_main::{create_main_mcb, Main};
use mcb::mcb_node::{create_node_mcb, CommandType, Node, Request};

use mcb::IntfResult::*;

use std::thread;
use std::{sync::mpsc, sync::mpsc::Receiver, sync::mpsc::Sender};

use float_eq::assert_float_eq;

struct NodeThread<T> {
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}
struct MainThread<T> {
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}

struct NodeThreadWrongCRC<T>(NodeThread<T>);
struct MainThreadWrongCRC<T>(MainThread<T>);

fn create_mainnodethread<T>() -> (NodeThread<T>, MainThread<T>) {
    let (mtx, mrx) = mpsc::channel();
    let (stx, srx) = mpsc::channel();

    (
        NodeThread {
            tx_channel: stx,
            rx_channel: mrx,
        },
        MainThread {
            tx_channel: mtx,
            rx_channel: srx,
        },
    )
}

fn create_main_wrongnode_thread<T>() -> (NodeThreadWrongCRC<T>, MainThread<T>) {
    let (mtx, mrx) = mpsc::channel();
    let (stx, srx) = mpsc::channel();

    (
        NodeThreadWrongCRC(NodeThread {
            tx_channel: stx,
            rx_channel: mrx,
        }),
        MainThread {
            tx_channel: mtx,
            rx_channel: srx,
        },
    )
}

fn create_wrongmain_node_thread<T>() -> (NodeThread<T>, MainThreadWrongCRC<T>) {
    let (mtx, mrx) = mpsc::channel();
    let (stx, srx) = mpsc::channel();

    (
        NodeThread {
            tx_channel: stx,
            rx_channel: mrx,
        },
        MainThreadWrongCRC(MainThread {
            tx_channel: mtx,
            rx_channel: srx,
        }),
    )
}

impl PhysicalInterface for NodeThread<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&mut self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.tx_channel.send(msg).unwrap();
        Ok(Success)
    }
    fn raw_read(&mut self) -> Result<IntfResult, IntfError> {
        let msg = self.rx_channel.recv().unwrap();
        Ok(Data(Box::new(msg)))
    }
}

impl PhysicalInterface for MainThread<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&mut self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.tx_channel.send(msg).unwrap();
        Ok(Success)
    }

    fn raw_read(&mut self) -> Result<IntfResult, IntfError> {
        let msg = self.rx_channel.recv().unwrap();
        Ok(Data(Box::new(msg)))
    }
}

impl PhysicalInterface for NodeThreadWrongCRC<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&mut self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.0.tx_channel.send(msg).unwrap();
        Ok(Success)
    }
    fn raw_read(&mut self) -> Result<IntfResult, IntfError> {
        let msg = self.0.rx_channel.recv().unwrap();
        Ok(Data(Box::new(msg)))
    }

    fn crc_checksum(&mut self, _frame: &[u16]) -> u16 {
        let result: u16 = 0u16;
        result
    }
}

impl PhysicalInterface for MainThreadWrongCRC<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&mut self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.0.tx_channel.send(msg).unwrap();
        Ok(Success)
    }

    fn raw_read(&mut self) -> Result<IntfResult, IntfError> {
        let msg = self.0.rx_channel.recv().unwrap();
        Ok(Data(Box::new(msg)))
    }

    fn crc_checksum(&mut self, _frame: &[u16]) -> u16 {
        let result: u16 = 0u16;
        result
    }
}

fn init_main(
    main_thread: MainThread<[u16; MAX_FRAME_SIZE]>,
) -> Main<Config, MainThread<[u16; MAX_FRAME_SIZE]>> {
    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread), ExtMode::Extended);

    mcb_main_test.init()
}

fn init_wrong_main(
    main_thread: MainThreadWrongCRC<[u16; MAX_FRAME_SIZE]>,
) -> Main<Config, MainThreadWrongCRC<[u16; MAX_FRAME_SIZE]>> {
    let mcb_main_test: Main<Init, MainThreadWrongCRC<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread), ExtMode::Extended);

    mcb_main_test.init()
}

fn init_node(
    node_thread: NodeThread<[u16; MAX_FRAME_SIZE]>,
) -> Node<Config, NodeThread<[u16; MAX_FRAME_SIZE]>> {
    let mcb_node_test: Node<Init, NodeThread<[u16; MAX_FRAME_SIZE]>> =
        create_node_mcb(Some(node_thread), ExtMode::Extended);

    mcb_node_test.init()
}

fn init_wrong_node(
    node_thread: NodeThreadWrongCRC<[u16; MAX_FRAME_SIZE]>,
) -> Node<Config, NodeThreadWrongCRC<[u16; MAX_FRAME_SIZE]>> {
    let mcb_node_test: Node<Init, NodeThreadWrongCRC<[u16; MAX_FRAME_SIZE]>> =
        create_node_mcb(Some(node_thread), ExtMode::Extended);

    mcb_node_test.init()
}

fn get_request(
    node_cfg: &mut Node<Config, NodeThread<[u16; MAX_FRAME_SIZE]>>,
) -> Result<Request, IntfError> {
    let mut is_ready = node_cfg.listen();

    loop {
        match is_ready {
            Ok(IntfResult::Empty) => {
                is_ready = node_cfg.listen();
            }
            _ => break,
        }
    }

    let request = match node_cfg.read() {
        Ok(request) => request,
        Err(IntfError::Crc) => return Err(IntfError::Crc),
        _ => {
            panic!("Something wrong");
        }
    };

    Ok(request)
}

fn get_wrong_request(
    node_cfg: &mut Node<Config, NodeThreadWrongCRC<[u16; MAX_FRAME_SIZE]>>,
) -> Result<Request, IntfError> {
    let mut is_ready = node_cfg.listen();

    loop {
        match is_ready {
            Ok(IntfResult::Empty) => {
                is_ready = node_cfg.listen();
            }
            _ => break,
        }
    }

    let request = match node_cfg.read() {
        Ok(request) => request,
        Err(IntfError::Crc) => return Err(IntfError::Crc),
        _ => {
            panic!("Something wrong");
        }
    };

    Ok(request)
}

#[test]
fn test_std_read_u8() {
    const ADDRESS: u16 = 10u16;
    const DATA: u8 = 0xA5u8;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_u8(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg= init_main(main_thread);
    let result = mcb_main_cfg.read_u8(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_u8() {
    const ADDRESS: u16 = 10u16;
    const DATA: u8 = 0xA5u8;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_u8(&request) == DATA {
            let _ = node_cfg.write_u8(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u8(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_u16() {
    const ADDRESS: u16 = 10u16;
    const DATA: u16 = 0xA5A5u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_u16(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u16(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_u16() {
    const ADDRESS: u16 = 10u16;
    const DATA: u16 = 0xA5A5u16;

    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_u16(&request) == DATA {
            let _ = node_cfg.write_u16(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u16(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_u32() {
    const ADDRESS: u16 = 10u16;
    const DATA: u32 = 0xA5A5A5A5u32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_u32(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u32(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_u32() {
    const ADDRESS: u16 = 10u16;
    const DATA: u32 = 0xA5A5A5A5u32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_u32(&request) == DATA {
            let _ = node_cfg.write_u32(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u32(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_u64() {
    const ADDRESS: u16 = 10u16;
    const DATA: u64 = 0xA5A5A5A5A5A5A5A5u64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_u64(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u64(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_u64() {
    const ADDRESS: u16 = 10u16;
    const DATA: u64 = 0xA5A5A5A5A5A5A5A5u64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_u64(&request) == DATA {
            let _ = node_cfg.write_u64(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u64(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_i8() {
    const ADDRESS: u16 = 10u16;
    const DATA: i8 = 0xA5u8 as i8;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_i8(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_i8(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_i8() {
    const ADDRESS: u16 = 10u16;
    const DATA: i8 = 0xA5u8 as i8;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_i8(&request) == DATA {
            let _ = node_cfg.write_i8(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_i8(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_i16() {
    const ADDRESS: u16 = 10u16;
    const DATA: i16 = 0xA5A5u16 as i16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_i16(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_i16(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_i16() {
    const ADDRESS: u16 = 10u16;
    const DATA: i16 = 0xA5A5u16 as i16;

    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_i16(&request) == DATA {
            let _ = node_cfg.write_i16(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_i16(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_i32() {
    const ADDRESS: u16 = 10u16;
    const DATA: i32 = 0xA5A5A5A5u32 as i32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_i32(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_i32(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_i32() {
    const ADDRESS: u16 = 10u16;
    const DATA: i32 = 0xA5A5A5A5u32 as i32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_i32(&request) == DATA {
            let _ = node_cfg.write_i32(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_i32(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_i64() {
    const ADDRESS: u16 = 10u16;
    const DATA: i64 = 0xA5A5A5A5A5A5A5A5u64 as i64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_i64(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_i64(ADDRESS);

    assert!(matches!(result, Ok(DATA)));
}

#[test]
fn test_std_write_i64() {
    const ADDRESS: u16 = 10u16;
    const DATA: i64 = 0xA5A5A5A5A5A5A5A5u64 as i64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_i64(&request) == DATA {
            let _ = node_cfg.write_i64(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_i64(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_f32() {
    const ADDRESS: u16 = 10u16;
    const DATA: f32 = 1.0 as f32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_f32(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_f32(ADDRESS);

    assert_float_eq!(result.unwrap() - DATA, 0.0, abs <= f32::EPSILON);
}

#[test]
fn test_std_write_f32() {
    const ADDRESS: u16 = 10u16;
    const DATA: f32 = 1.0 as f32;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_f32(&request) == DATA {
            let _ = node_cfg.write_f32(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_f32(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_std_read_f64() {
    const ADDRESS: u16 = 10u16;
    const DATA: f64 = 1.0 as f64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.write_f64(request.address, DATA) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_f64(ADDRESS);

    assert_float_eq!(result.unwrap() - DATA, 0.0, abs <= f64::EPSILON);
}

#[test]
fn test_std_write_f64() {
    const ADDRESS: u16 = 10u16;
    const DATA: f64 = 1.0 as f64;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        if node_cfg.get_data_f64(&request) == DATA {
            let _ = node_cfg.write_f64(request.address, DATA);
        } else {
            let _ = node_cfg.error(request.address, 0x0u32);
        }
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_f64(ADDRESS, DATA);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_main_write_out_of_index_address() {
    const ADDRESS: u16 = 0x1000u16;
    let (_node_thread, main_thread) = create_mainnodethread();

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u8(ADDRESS, 1u8);

    assert!(matches!(result, Err(IntfError::AddressOutOfIndex)));
}

#[test]
fn test_main_read_out_of_index_address() {
    const ADDRESS: u16 = 0x1000u16;
    let (_node_thread, main_thread) = create_mainnodethread();

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u8(ADDRESS);

    assert!(matches!(result, Err(IntfError::AddressOutOfIndex)));
}

#[test]
fn test_main_write_unexistent_register() {
    const ADDRESS: u16 = 0x0100u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.error(request.address, 0x80005000u32) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.write_u8(ADDRESS, 1u8);

    assert!(matches!(result, Err(IntfError::Access(0x80005000u32))));
}

#[test]
fn test_main_read_unexistent_register() {
    const ADDRESS: u16 = 0x0250u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = match get_request(&mut node_cfg) {
            Ok(request) => request,
            _ => {
                panic!("Something wrong");
            }
        };

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.error(request.address, 0x80005000u32) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u8(ADDRESS);

    assert!(matches!(result, Err(IntfError::Access(0x80005000u32))));
}

#[test]
fn test_main_wrong_node_crc() {
    const ADDRESS: u16 = 10u16;
    let (node_thread, main_thread) = create_main_wrongnode_thread();

    thread::spawn(move || {
        let mut node_cfg = init_wrong_node(node_thread);
        match get_wrong_request(&mut node_cfg) {
            Err(IntfError::Crc) => (),
            _ => {
                panic!("Something wrong");
            }
        }

        let _result = match node_cfg.write_u8(ADDRESS, 1u8) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_main(main_thread);
    let result = mcb_main_cfg.read_u8(ADDRESS);

    assert!(matches!(result, Err(IntfError::Crc)));
}

#[test]
fn test_node_wrong_main_crc() {
    const ADDRESS: u16 = 10u16;
    let (node_thread, main_thread) = create_wrongmain_node_thread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        match get_request(&mut node_cfg) {
            Err(IntfError::Crc) => (),
            _ => {
                panic!("Something wrong");
            }
        };

        let _result = match node_cfg.write_u8(ADDRESS, 1u8) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mut mcb_main_cfg = init_wrong_main(main_thread);
    let result = mcb_main_cfg.read_u8(ADDRESS);
    assert!(matches!(result, Err(IntfError::Crc)));
}
