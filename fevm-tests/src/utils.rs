use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_ipld_encoding::{strict_bytes, BytesDe};
use test_vm::{util::create_accounts, VM};
use fil_actors_runtime::{EAM_ACTOR_ADDR};
use fvm_shared::{econ::TokenAmount, address::Address};

use serde::{Deserialize, Serialize};
use num_traits::Zero;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct ContractParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

pub fn deploy_contract<'a>(store: &'a MemoryBlockstore, bytecode: Vec<u8>) -> Option<(VM<'a>, Address, Address)>{
    let vm = VM::new_with_singletons(store);
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
        return None;
    }

    let create_return: fil_actor_eam::Create2Return =
        create_result.ret.deserialize().expect("failed to decode results");
    
    return Some((vm, account, create_return.robust_address));
}

pub fn invoke_contract(vm: &VM, from: Address, to: Address, contract_params: Vec<u8>) -> Option<Vec<u8>> {
    let params = Some(ContractParams(contract_params.to_vec()));
    println!("=================================");
    println!("params: {:?}", params);

    let call_result = vm
        .apply_message(
            from,
            to,
            TokenAmount::zero(),
            fil_actor_evm::Method::InvokeContract as u64,
            params,
        )
        .unwrap();

    if !call_result.code.is_success() {
        println!("failed to call the new actor {}", call_result.message);
        return None;
    }

    let BytesDe(return_value) = call_result.ret.deserialize().expect("failed to deserialize results");
    
    // check return_value of contract call
    
    println!("return: {:?}", &return_value);
    println!("=================================");

    Some(return_value)
}