use mcb_rust::{OperationResult, McbData};

#[test]
fn test_initialization() {
    assert_eq!(mcb_rust::init(), true);
}

#[test]
fn test_read(){
    let reg:  McbData ={McbData{data: 0u64}};
    let result: bool;

    match mcb_rust::read(0u16, reg) {
        OperationResult::Success => result = true,
        OperationResult::Fail => result = false,
    }
    assert_eq!(result, true);
}
