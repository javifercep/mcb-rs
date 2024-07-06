
![workflow](https://github.com/javifercep/Mcb-for-rust/actions/workflows/rust.yml/badge.svg)

# Motion Control Bus - Rust

 This is a personal project for learning the basics of Rust.

 ## Motivation
 Learning how to use Rust in embedded systems and learn about the "safe" features the language offers by default.

 ## What does it do?

`mcb-rs` implements the Motion Control Bus protocol:
  * [Motion Control Bus](https://drives.novantamotion.com/eve-core/mcb-overview)

This library works over other protocols such as Ethernet,
SPI or USB. It is designed for embedded system where the
hardware abstraction needs to be created. This means that
this library requires the implementation of a struct implementing
the [`PhysicalInterface`] trait.

### Examples

```rust
use mcb::mcb_main::{create_main_mcb, Main};
use mcb::{Config, Init, IntfError, IntfResult, PhysicalInterface, MAX_FRAME_SIZE};
use mcb::IntfResult::*;

struct NewInterface;

impl PhysicalInterface for NewInterface {
    
    fn raw_write(&self, frame: &[u16]) -> Result<IntfResult, IntfError> {
    // your implementation
    Ok(Success)
    }

    fn raw_read(&self) -> Result<IntfResult, IntfError> {
        // ignore this block. Created to pass cargo test --doc
        let mut msg = [0u16; MAX_FRAME_SIZE];
        msg[1] = 166;
        msg[2] = 1;
        msg[6] = 17282;
        // end of ignore block
        // your implementation
        Ok(Data(Box::new(msg)))
    }

    fn is_data2read(&self) -> Result<IntfResult, IntfError> {
        // your implementation
        Ok(Success)
    }
}

fn main() {
    let interface: NewInterface = NewInterface;

    let mcb_main = create_main_mcb(Some(interface));
    
    let mut mcb_main_cfg = mcb_main.init();
    let result = mcb_main_cfg.writeu8(0x0Au16, 1u8);

    assert!(matches!(result, Ok(IntfResult::Success)));
}
```

This crate contains both modules
 * Main
 * Node

The main module is the expected to be used over SPI with the drives implementing these protocols:
 * [Capitan CORE](https://drives.novantamotion.com/cap-core/)
 * [Everest CORE](https://drives.novantamotion.com/eve-core/)

The node module is expected to be used for bridge applications such as Turonet
 * [Hardware](https://github.com/javifercep/Turonet)
 * [Firmware](https://github.com/javifercep/turonet-rs)
