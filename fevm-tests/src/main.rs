use fvm_ipld_blockstore::MemoryBlockstore;

use test_vm::{util::create_accounts, VM};
use fvm_shared::{econ::TokenAmount};

use serde::{Deserialize};
use std::{fs};

mod vm_utils;
mod abi_utils;
mod test_runner;

use test_runner::ContractTestRunner;

#[derive(Debug, Deserialize)]
struct ContractTests {
    bytecode_file_path: String,
    abi_file_path: String,
    tests: Vec<ContractTest>,
}

#[derive(Debug, Deserialize)]
struct ContractTest {
    function_name_or_signature: String,
    inputs: Vec<String>,
    // outputs: Vec<ContractTestOutput>,
}

#[derive(Debug, Deserialize)]
struct ContractTestOutput {

}

fn main() {
    // setup
    let store = MemoryBlockstore::new();
    let vm = VM::new_with_singletons(&store);
    let user_addr = create_accounts(&vm, 1, TokenAmount::from_whole(10_000))[0];

    let contract_test_json_paths = fs::read_dir("./tests/").unwrap();
    for dir_entry in contract_test_json_paths {

        // load a test json
        let path = dir_entry.unwrap().path();
        let contract_tests_str = fs::read_to_string(&path).expect("Unable to read contract tests json file");
        let contract_tests: ContractTests = serde_json::from_str(&contract_tests_str).expect("Unable deserialize contract tests json");
        
        let test_runner = ContractTestRunner::new(&vm, &contract_tests.bytecode_file_path, &contract_tests.abi_file_path, user_addr);

        // run all tests for this json
        println!("\nRunning: {}", path.display());
        for contract_test in contract_tests.tests {
            let inputs: Vec<&str> = contract_test.inputs.iter().map(String::as_str).collect();
            let outputs = test_runner.invoke_function(user_addr, &contract_test.function_name_or_signature, &inputs);
            
            // let inputs_str = inputs.join(", ");
            println!("{}({}) = {:?}", contract_test.function_name_or_signature, inputs.join(", "), outputs);
        }
    }
}