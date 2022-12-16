use fil_actor_evm::interpreter::{address::EthAddress, output::Outcome, Output, execute, System, Bytecode, ExecutionState, U256};
use bytes::Bytes;
use fil_actors_runtime::test_utils::MockRuntime;
use etk_asm::ingest::Ingest;

fn main() {
    let bytecode_body = r#"
    %push(0xdeadbeef)
    push1 0x00
    mstore
    push1 0x04
    push1 0x1c # skip top 28 bytes
    revert
    "#;

    let bytecode = Bytecode::new(to_bytecode("test", "", bytecode_body).unwrap());
    
    let output = simple_run_bytecode(&bytecode);
    let _res = match output.outcome {
        Outcome::Return => {
            println!("Return");
        }
        Outcome::Revert => {
            println!("Revert");
        },
        Outcome::Delete => {
            println!("Delete");
        },
    };
}

fn simple_run_bytecode(bytecode: &Bytecode) -> Output {
    let mock_eth_addr = mock_eth_addr();
    
    // mock fvm
    let mut rt = MockRuntime::default();
    // abstraction layer between fvm and evm
    let mut system = System::create(&mut rt).unwrap();
    // evm
    let mut exec_state = ExecutionState::new(mock_eth_addr, mock_eth_addr, Bytes::new());
    let output = execute(bytecode, &mut exec_state, &mut system).unwrap();
    output
}

fn mock_eth_addr() -> EthAddress {
    const TYPE_PADDING: &[u8] = &[0; 12]; // padding (12 bytes)
    const ID_ADDRESS_MARKER: &[u8] = &[0xff]; // ID address marker (1 byte)
    const GOOD_ADDRESS_PADDING: &[u8] = &[0, 11]; // padding for inner u64 (11 bytes)
    const REST: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
    let eth_addr_parts = [TYPE_PADDING, ID_ADDRESS_MARKER, GOOD_ADDRESS_PADDING, REST];
    let eth_addr = EthAddress::try_from(U256::from(eth_addr_parts.concat().as_slice())).unwrap();
    eth_addr
}

// converts from asm to Vec<u8>
fn to_bytecode(name: &str, init: &str, body: &str) -> Result<Vec<u8>, etk_asm::ingest::Error> {
    let mut body_code = Vec::new();
    let mut ingest_body = Ingest::new(&mut body_code);
    ingest_body.ingest(name, body)?;

    let mut init_code = Vec::new();
    let mut ingest_init = Ingest::new(&mut init_code);
    ingest_init.ingest(name, init)?;

    let mut contract_code = Vec::new();
    contract_code.append(&mut init_code);
    contract_code.append(&mut body_code);
    
    Ok(contract_code)
}
