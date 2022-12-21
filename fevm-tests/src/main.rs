use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_ipld_encoding::{strict_bytes, BytesDe};
use test_vm::{util::create_accounts, VM};
use fil_actors_runtime::{EAM_ACTOR_ADDR};
use fvm_shared::{econ::TokenAmount};

use serde::{Deserialize, Serialize};
use num_traits::Zero;
use std::fs;
use std::path::Path;

fn main() {
    simple_contract_example();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct ContractParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

fn simple_contract_example() {
    let contract_file_path = "./contracts/Recursive.hex";
    if !Path::new(&contract_file_path).exists() {
        println!("couldn't find contract at: {}", contract_file_path);
        return;
    }

    let bytecode_str = fs::read_to_string(contract_file_path).expect("unable to read contract bytecode file");
    let bytecode = hex::decode(bytecode_str).unwrap();

    // which contract func to call 
    // enter func hash from Recursive.signatures
    let mut params = vec![];
    params.append(&mut hex::decode("e97dcb62").unwrap());

    invoke_contract(bytecode, params);
}

fn invoke_contract(bytecode: Vec<u8>, contract_params: Vec<u8>) {
    let store = MemoryBlockstore::new();
    let vm = VM::new_with_singletons(&store);
    let account = create_accounts(&vm, 1, TokenAmount::from_whole(10_000))[0];

    let create_result = vm
        .apply_message(
            account,
            EAM_ACTOR_ADDR,
            TokenAmount::zero(),
            fil_actor_eam::Method::Create2 as u64,
            Some(fil_actor_eam::Create2Params { initcode: bytecode, salt: [0u8; 32] }),
        )
        .unwrap();


    if !create_result.code.is_success() {
        println!("failed to create the new actor {}", create_result.message);
        return;
    }

    let create_return: fil_actor_eam::Create2Return =
        create_result.ret.deserialize().expect("failed to decode results");

    let params = Some(ContractParams(contract_params.to_vec()));
    println!("{:?}", params);

    let call_result = vm
        .apply_message(
            account,
            create_return.robust_address,
            TokenAmount::zero(),
            fil_actor_evm::Method::InvokeContract as u64,
            Some(ContractParams(contract_params.to_vec())),
        )
        .unwrap();

    if !call_result.code.is_success() {
        println!("failed to call the new actor {}", create_result.message);
        return;
    }

    let BytesDe(return_value) = call_result.ret.deserialize().expect("failed to deserialize results");
    
    // check return_value of contract call
    
    println!("{:?}", return_value);
}
