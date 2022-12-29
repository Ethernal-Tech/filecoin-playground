use fvm_ipld_encoding::{strict_bytes, BytesDe};
use test_vm::{VM};
use fil_actors_runtime::{EAM_ACTOR_ADDR};
use fvm_shared::{econ::TokenAmount, address::Address};

use serde::{Deserialize, Serialize};
use num_traits::Zero;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct ContractParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

pub fn vm_deploy_contract<'a>(vm: &'a VM, bytecode: Vec<u8>, deployer_addr: Address) -> Option<Address>{
    let create_result = vm
        .apply_message(
            deployer_addr,
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
    
    return Some(create_return.robust_address);
}

pub fn vm_invoke_contract(vm: &VM, from: Address, to: Address, contract_params: Vec<u8>) -> Option<Vec<u8>> {
    let params = Some(ContractParams(contract_params.to_vec()));
    // println!("=================================");
    // println!("params: {:?}", params);

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
    
    // println!("return: {:?}", &return_value);
    // println!("=================================");

    Some(return_value)
}