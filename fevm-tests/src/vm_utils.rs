use fvm_ipld_encoding::{strict_bytes, BytesDe};
use test_vm::{VM};
use fil_actors_runtime::{EAM_ACTOR_ADDR};
use fvm_shared::{econ::TokenAmount, address::Address};

use serde::{Deserialize, Serialize};
use num_traits::Zero;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct ContractParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

pub fn vm_deploy_contract(vm: &VM, bytecode: Vec<u8>, deployer_addr: Address) -> Result<fil_actor_eam::Return>{
    let result = vm
        .apply_message(
            deployer_addr,
            EAM_ACTOR_ADDR,
            TokenAmount::zero(),
            fil_actor_eam::Method::Create2 as u64,
            Some(fil_actor_eam::Create2Params { initcode: bytecode, salt: [0u8; 32] }),
        );

    if let Err(create_err) = result {
        return Err(anyhow!("failed to create the new actor: {:?}", create_err));
    }

    let create_result = result.unwrap();

    if !create_result.code.is_success() {
        return Err(anyhow!("failed to create the new actor: {}", create_result.message));
    }

    let create_return_result: Result<fil_actor_eam::Create2Return, fvm_ipld_encoding::Error> =
        create_result.ret.deserialize();
       
    if let Err(_err) = create_return_result {
        return Err(anyhow!("failed to decode create results"));
    }
    
    return Ok(create_return_result.unwrap());
}

pub fn vm_invoke_contract(vm: &VM, from: Address, to: Address, contract_params: Vec<u8>) -> Result<Vec<u8>> {
    let params = Some(ContractParams(contract_params.to_vec()));

    let result = vm
        .apply_message(
            from,
            to,
            TokenAmount::zero(),
            fil_actor_evm::Method::InvokeContract as u64,
            params,
        );

    if let Err(call_err) = result {
        return Err(anyhow!("failed to invoke an actor: {:?}", call_err));
    }

    let call_result = result.unwrap();

    if !call_result.code.is_success() {
        return Err(anyhow!("failed to invoke an actor: {}", call_result.message));
    }

    let deserialize_result = call_result.ret.deserialize();
    match deserialize_result {
        Ok(BytesDe(return_value)) => Ok(return_value),
        _ => Err(anyhow!("failed to deserialize invoke results")),
    }
}