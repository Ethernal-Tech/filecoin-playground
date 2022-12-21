use fil_actor_evm::interpreter::{address::EthAddress, output::Outcome, Output, execute, System, Bytecode, ExecutionState, U256};
use bytes::Bytes;
use fil_actors_runtime::test_utils::MockRuntime;
use etk_asm::ingest::Ingest;

mod mock_contract_vm;

use fvm_ipld_blockstore::MemoryBlockstore;
use mock_contract_vm::mock_contract_runtime;
use test_vm::{VM};
use fil_actors_runtime::runtime::{Policy};

fn main() {
    simple_contract_bytecode_example();
}

fn simple_contract_bytecode_example() {
    // recursive.sol
    let bytecode_str = "608060405234801561001057600080fd5b506102ac806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063e97dcb621461003b578063ea0f66b114610059575b600080fd5b610043610077565b60405161005091906101fd565b60405180910390f35b61006161017f565b60405161006e91906101fd565b60405180910390f35b60008060009054906101000a900460ff1615610096576001905061017c565b60016000806101000a81548160ff02191690831515021790555060003073ffffffffffffffffffffffffffffffffffffffff1663ea0f66b16040518163ffffffff1660e01b81526004016020604051808303816000875af11580156100ff573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906101239190610249565b905060008163ffffffff161461013c578091505061017c565b60008054906101000a900460ff1661015857600491505061017c565b600060019054906101000a900460ff1661017657600591505061017c565b60009150505b90565b60008060009054906101000a900460ff1661019d57600290506101db565b600060019054906101000a900460ff16156101bb57600390506101db565b6001600060016101000a81548160ff021916908315150217905550600090505b90565b600063ffffffff82169050919050565b6101f7816101de565b82525050565b600060208201905061021260008301846101ee565b92915050565b600080fd5b610226816101de565b811461023157600080fd5b50565b6000815190506102438161021d565b92915050565b60006020828403121561025f5761025e610218565b5b600061026d84828501610234565b9150509291505056fea264697066735822122048dc6be27b10b6aea4c61982bf8f8c3cd96348d729cc42a0daf77b2a2675d9f464736f6c63430008110033";
    let bytecode = hex::decode(bytecode_str).unwrap();
    
    // which contract func to call 
    // let mut params = vec![];
    // enter func hash from recursive.sol 
    // params.append(&mut hex::decode("e97dcb62").unwrap());

    let output = simple_run_contract_bytecode(bytecode/*, params*/);
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

fn simple_run_contract_bytecode(bytecode: Vec<u8>/*, params: Vec<u8>*/) -> Output {
    let store = MemoryBlockstore::new();
    let vm = VM::new_with_singletons(&store);
    let policy = Policy::default();
    let mock_eth_addr = mock_eth_addr();
    
    // mock fvm
    let mut rt = mock_contract_runtime(&vm, &policy, bytecode);
    // abstraction layer between fvm and evm
    let mut system = System::load(&mut rt).unwrap();
    let loaded_bytecode = match system.load_bytecode().unwrap() {
        Some(l_bytecode) => l_bytecode,
        None => panic!("no loaded bytecode"),
    };
    // evm
    let mut exec_state = ExecutionState::new(mock_eth_addr, mock_eth_addr, Bytes::new()/*params.into()*/);
    let output = execute(&loaded_bytecode, &mut exec_state, &mut system).unwrap();
    output
}

fn simple_asm_bytecode_example() {
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
