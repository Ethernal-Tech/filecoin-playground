use fvm_ipld_blockstore::MemoryBlockstore;

use test_vm::{VM};
use fvm_shared::{address::Address};

use ethabi::{Contract, token::{Token}};

use std::fs;
use std::path::Path;

use crate::vm_utils::{vm_deploy_contract, vm_invoke_contract};
use crate::abi_utils::{abi_load_contract, abi_load_function, abi_encode, abi_decode};

fn invoke_function(
    vm: &VM, from: Address, to: Address,
    contract_abi: &Contract, function_name_or_signature: &str, inputs: &[&str]
) -> Vec<Token> {

    let function_abi = abi_load_function(&contract_abi, function_name_or_signature).unwrap();
    let encoded_params = abi_encode(&function_abi, inputs).unwrap();

    let result = vm_invoke_contract(&vm, from, to, encoded_params).unwrap();

    let decoded_output = abi_decode(&function_abi, &result).unwrap();

    let inputs_str = inputs.join(", ");
    println!("{function_name_or_signature}({inputs_str}) = {:?}", decoded_output);

    decoded_output
}

pub fn io_tests_contract_example() {
    let contract_path = "./contracts/IOTests";
    let contract_hex_file_path = format!("{contract_path}.hex");
    let contract_abi_file_path = format!("{contract_path}.abi");

    if !Path::new(&contract_hex_file_path).exists() || !Path::new(&contract_abi_file_path).exists() {
        println!("couldn't find contract hex or abi files at: {}", contract_path);
        return;
    }

    let store = MemoryBlockstore::new();
    let bytecode_str = fs::read_to_string(contract_hex_file_path).expect("unable to read contract bytecode file");
    let bytecode = hex::decode(bytecode_str).unwrap();
    let (vm, user_addr, contract_addr) = vm_deploy_contract(&store, bytecode).unwrap();
	
    let contract_abi = abi_load_contract(&contract_abi_file_path).unwrap();

    // e7c0150a: uints0in1out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints0in1out", &[]);

    // 5b4c1f19: uints0in2out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints0in2out", &[]);

    // 127c7f70: uints0in3out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints0in3out", &[]);

    // 1271baa2: strings0in1out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings0in1out", &[]);

    // efda80ff: strings0in2out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings0in2out", &[]);

    // 4d09eb9d: strings0in3out()
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings0in3out", &[]);

    // 96976d81: uints1in1out(uint32)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints1in1out", &["5"]);

    // b67ce6ed: uints2in2out(uint32,uint32)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints2in2out", &["5", "10"]);

    // 58e45590: uints2in1out(uint32,uint32)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "uints2in1out", &["5", "10"]);

    // a95e3a72: strings1in1out(string)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings1in1out", &["1"]);

    // ba9512c1: strings2in2out(string,string)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings2in2out", &["1", "5"]);

    // 65b9c0e7: strings2in1out(string,string)
    invoke_function(&vm, user_addr, contract_addr, &contract_abi, "strings2in1out", &["1", "5"]);
}