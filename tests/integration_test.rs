use mcb::mcb_main::{create_main_mcb, Main};
use mcb::{Config, Init, IntfError, IntfResult, PhysicalInterface, MAX_FRAME_SIZE};

use mcb::mcb_node::{create_node_mcb, CommandType, Node, Request};

use mcb::{IntfError::*, IntfResult::*};

use std::thread;
use std::{sync::mpsc, sync::mpsc::Receiver, sync::mpsc::Sender};

struct NodeThread<T> {
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}
struct MainThread<T> {
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}

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

impl PhysicalInterface for NodeThread<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.tx_channel.send(msg).unwrap();
        Ok(Success)
    }
    fn raw_read(&self) -> Result<IntfResult, IntfError> {
        let msg = self.rx_channel.recv().unwrap();
        Ok(Data(msg))
    }

    fn is_data2read(&self) -> Result<IntfResult, IntfError> {
        Ok(Success)
    }
}

impl PhysicalInterface for MainThread<[u16; MAX_FRAME_SIZE]> {
    fn raw_write(&self, frame: &[u16]) -> Result<IntfResult, IntfError> {
        let mut msg = [0u16; MAX_FRAME_SIZE];

        msg[..frame.len()].copy_from_slice(frame);

        self.tx_channel.send(msg).unwrap();
        Ok(Success)
    }

    fn raw_read(&self) -> Result<IntfResult, IntfError> {
        let msg = self.rx_channel.recv().unwrap();
        Ok(Data(msg))
    }

    fn is_data2read(&self) -> Result<IntfResult, IntfError> {
        Ok(Success)
    }
}

fn init_node(
    node_thread: NodeThread<[u16; MAX_FRAME_SIZE]>,
) -> Node<Config, NodeThread<[u16; MAX_FRAME_SIZE]>> {
    let mcb_node_test: Node<Init, NodeThread<[u16; MAX_FRAME_SIZE]>> =
        create_node_mcb(Some(node_thread));

    mcb_node_test.init()
}

fn get_request(node_cfg: &mut Node<Config, NodeThread<[u16; MAX_FRAME_SIZE]>>) -> Request {
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
        _ => {
            panic!("Something wrong");
        }
    };

    request
}

#[test]
fn test_main_std_read() {
    const ADDRESS: u16 = 10u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = get_request(&mut node_cfg);

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Read) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.writeu8(request.address, 1u8) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.readu8(ADDRESS);

    assert!(matches!(result, Ok(1u8)));
}

#[test]
fn test_main_std_write() {
    const ADDRESS: u16 = 10u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = get_request(&mut node_cfg);

        if request.address != ADDRESS {
            panic!("Something wrong");
        }
        if !matches!(request.command, CommandType::Write) {
            panic!("Something wrong");
        }

        let _result = match node_cfg.writeu8(request.address, 1u8) {
            Ok(Success) => true,
            _ => false,
        };
    });

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.writeu8(ADDRESS, 1u8);

    assert!(matches!(result, Ok(IntfResult::Success)));
}

#[test]
fn test_main_write_out_of_index_address() {
    const ADDRESS: u16 = 0x1000u16;
    let (_node_thread, main_thread) = create_mainnodethread();

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.writeu8(ADDRESS, 1u8);

    assert!(matches!(result, Err(IntfError::AddressOutOfIndex)));
}

#[test]
fn test_main_read_out_of_index_address() {
    const ADDRESS: u16 = 0x1000u16;
    let (_node_thread, main_thread) = create_mainnodethread();

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.readu8(ADDRESS);

    assert!(matches!(result, Err(IntfError::AddressOutOfIndex)));
}

#[test]
fn test_main_write_unexistent_register() {
    const ADDRESS: u16 = 0x0100u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = get_request(&mut node_cfg);

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

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.writeu8(ADDRESS, 1u8);

    assert!(matches!(result, Err(IntfError::Access(0x80005000u32))));
}

#[test]
fn test_main_read_unexistent_register() {
    const ADDRESS: u16 = 0x0250u16;
    let (node_thread, main_thread) = create_mainnodethread();

    thread::spawn(move || {
        let mut node_cfg = init_node(node_thread);
        let request = get_request(&mut node_cfg);

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

    let mcb_main_test: Main<Init, MainThread<[u16; MAX_FRAME_SIZE]>> =
        create_main_mcb(Some(main_thread));
    let mut mcb_main_cfg = mcb_main_test.init();
    let result = mcb_main_cfg.readu8(ADDRESS);

    assert!(matches!(result, Err(IntfError::Access(0x80005000u32))));
}
