use mcb::{
    create_master_mcb, create_slave_mcb, Init, Master, Slave, 
    Frame, PhysicalInterface, IntfResult, IntfError, 
};

use mcb::{
    IntfResult::*,
    IntfError::*,
};

use std::{
    sync::mpsc, sync::mpsc::Sender, sync::mpsc::Receiver
};
use std::thread;

struct SlaveThread<T> {
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}
struct MasterThread<T>{
    tx_channel: Sender<T>,
    rx_channel: Receiver<T>,
}

fn create_masterslavethread<T>() -> (SlaveThread<T>, MasterThread<T>) {
    let (mtx, mrx) = mpsc::channel();
    let (stx, srx) = mpsc::channel();

    (SlaveThread {
        tx_channel: stx,
        rx_channel: mrx,
    },
     MasterThread {
        tx_channel: mtx,
        rx_channel: srx,
     })


}

impl PhysicalInterface for SlaveThread<Frame> {
    fn initialize(&self) -> Result<IntfResult, IntfError> {
        Ok(Success)
    }
    fn raw_write(&self, frame: &Frame) -> Result<IntfResult, IntfError>  {
        Ok(Success)
    }
    fn raw_read(&self, frame: &Frame) -> Result<IntfResult, IntfError>  {
        match self.rx_channel.recv().unwrap() {
            _ => Ok(Success),
        }
    }
}

impl PhysicalInterface for MasterThread<Frame> {
    fn initialize(&self) -> Result<IntfResult, IntfError> {
        Ok(Success)
    }
    fn raw_write(&self, frame: &Frame) -> Result<IntfResult, IntfError>  {
        let msg = frame.clone();
        self.tx_channel.send(msg).unwrap();
        Ok(Success)
    }
    fn raw_read(&self, frame: &Frame) -> Result<IntfResult, IntfError>  {
        Ok(Success)
    }
}

#[test]
fn test_read() {
    let (slave_thread, master_thread) = create_masterslavethread();
    let mcb_slave_test: Slave<Init, SlaveThread<Frame>> = create_slave_mcb(Some(slave_thread));

    thread::spawn(move || {
        let mcb_master_test: Master<Init, MasterThread<Frame>> = create_master_mcb(Some(master_thread));
        let mut mcb_master_cfg = mcb_master_test.init();
        mcb_master_cfg.writeu8(0u16, 0u8);
        // commmented code to check test is working
    });

    let mut mcb_slave_cfg = mcb_slave_test.init();

    let mut result;

    match mcb_slave_cfg.readu8(0u16) {
        Ok(Success) => result = true,
        Err(Interface) => result = false,
        _ => result = false,
    }

    assert_eq!(result, true);
}
