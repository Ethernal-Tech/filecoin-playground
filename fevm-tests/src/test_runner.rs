use test_vm::{VM};
use fvm_shared::{address::Address};

use ethabi::{Contract, token::{Token}};

use std::fs;
use std::path::Path;

use crate::vm_utils::{vm_deploy_contract, vm_invoke_contract};
use crate::abi_utils::{abi_load_contract, abi_load_function, abi_encode, abi_decode};

pub struct ContractTestRunner<'a> {
    vm: &'a VM<'a>,
    contract_addr: Address,
    contract_abi: Contract,
}

impl<'a> ContractTestRunner<'a> {
    pub fn new(vm: &'a VM, contract_bytecode_path: &str, contract_abi_path: &str, deployer_addr: Address) -> Self {
        if !Path::new(&contract_bytecode_path).exists() || !Path::new(&contract_abi_path).exists() {
            println!("couldn't find contract hex or abi files at: {}; {}", contract_bytecode_path, contract_abi_path);
            panic!();
        }

        let bytecode_str = fs::read_to_string(contract_bytecode_path).expect("unable to read contract bytecode file");
        let bytecode = hex::decode(bytecode_str).unwrap();
        let contract_addr = vm_deploy_contract(&vm, bytecode, deployer_addr).unwrap();
        
        let contract_abi = abi_load_contract(&contract_abi_path).unwrap();

        Self {
            vm,
            contract_addr,
            contract_abi,
        }
    }

    pub fn invoke_function(&self, from: Address, function_name_or_signature: &str, inputs: &Vec<&str>) -> Vec<Token> {
        let function_abi = abi_load_function(&self.contract_abi, function_name_or_signature).unwrap();
        let encoded_params = abi_encode(&function_abi, inputs).unwrap();
    
        let result = vm_invoke_contract(self.vm, from, self.contract_addr, encoded_params).unwrap();
    
        let decoded_output = abi_decode(&function_abi, &result).unwrap();
        decoded_output
    }
}