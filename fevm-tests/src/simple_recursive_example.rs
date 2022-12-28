use fvm_ipld_blockstore::MemoryBlockstore;

use std::fs;
use std::path::Path;

use crate::vm_utils::{vm_deploy_contract, vm_invoke_contract};

pub fn simple_recursive_contract_example() {
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

    let store = MemoryBlockstore::new();
    let (vm, account, contract_addr) = vm_deploy_contract(&store, bytecode).unwrap();
    vm_invoke_contract(&vm, account, contract_addr, params);
}