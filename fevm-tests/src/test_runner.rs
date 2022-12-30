use test_vm::{VM};
use fvm_shared::{address::Address};

use ethabi::{Contract, token::{Token}, Function};

use std::{fs, sync::Mutex};
use std::path::Path;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::{vm_utils::{vm_deploy_contract, vm_invoke_contract}, abi_utils::{abi_tokenize_function_inputs, abi_tokenize_function_outputs}};
use crate::abi_utils::{abi_load_contract, abi_load_function, abi_encode, abi_decode};

pub struct ContractTestRunner<'a> {
    vm: &'a VM<'a>,
    contract_addr: Address,
    contract_abi: Contract,
    functions_map: Mutex<HashMap<String, Function>>,
}

impl<'a> ContractTestRunner<'a> {
    pub fn new(vm: &'a VM, contract_bytecode_path: &str, contract_abi_path: &str, deployer_addr: Address) -> Result<Self> {
        if !Path::new(&contract_bytecode_path).exists() || !Path::new(&contract_abi_path).exists() {
            return Err(anyhow!("couldn't find contract hex or abi files at: {}; {}", contract_bytecode_path, contract_abi_path));
        }

        let bytecode_str_result = fs::read_to_string(contract_bytecode_path);
        if let Err(_err) = bytecode_str_result {
            return Err(anyhow!("unable to read contract bytecode file"));
        }
        
        let bytecode_result = hex::decode(bytecode_str_result.unwrap());
        if let Err(_err) = bytecode_result {
            return Err(anyhow!("unable to hex::decode contract bytecode"));
        }
        
        let bytecode = bytecode_result.unwrap();
        let deployment_result = vm_deploy_contract(&vm, bytecode, deployer_addr)?;
        
        let contract_abi = abi_load_contract(&contract_abi_path)?;

        Ok(Self {
            vm,
            contract_addr: deployment_result.robust_address,
            contract_abi,
            functions_map: Mutex::new(HashMap::new()),
        })
    }

    pub fn invoke_function(&self, from: Address, function_name_or_signature: &str, inputs: &Vec<&str>) -> Result<Vec<Token>> {
        self.operate_on_function(function_name_or_signature, |function_abi| {
            let encoded_params = abi_encode(function_abi, inputs)?;
        
            let result = vm_invoke_contract(self.vm, from, self.contract_addr, encoded_params)?;
        
            let decoded_output = abi_decode(function_abi, &result)?;
            Ok(decoded_output)
        })
    }

    pub fn _tokenize_abi_function_inputs(&self, function_name_or_signature: &str, inputs: &Vec<&str>) -> Result<Vec<Token>> {
        self.operate_on_function(function_name_or_signature, |function_abi| {
            abi_tokenize_function_inputs(function_abi, inputs)
        })
    }

    pub fn tokenize_abi_function_outputs(&self, function_name_or_signature: &str, outputs: &Vec<&str>) -> Result<Vec<Token>> {
        self.operate_on_function(function_name_or_signature, |function_abi| {
            abi_tokenize_function_outputs(function_abi, outputs)
        })
    }

    fn operate_on_function<T, F>(&self, function_name_or_signature: &str, operate_f: F) -> Result<T>
        where F: FnOnce(&Function) -> Result<T>
    {
        let functions_map_result = self.functions_map.lock();
        if let Err(_err) = functions_map_result {
            return Err(anyhow!("failed to lock function cache"));
        }

        let mut functions_map = functions_map_result.unwrap();
        let insert_result: Result<()> = match functions_map.contains_key(function_name_or_signature) {
            true => Ok(()),
            false => {
                let function_abi = abi_load_function(&self.contract_abi, function_name_or_signature)?;
                functions_map.insert(function_name_or_signature.to_string(), function_abi);
                Ok(())
            }
        };

        if let Err(err) = insert_result {
            return Err(anyhow!("failed to insert into function cache: {}", err));
        }

        match functions_map.get(function_name_or_signature) {
            None => Err(anyhow!("failed to find function from function cache")),
            Some(function_abi) => operate_f(function_abi)
        }
    }
}