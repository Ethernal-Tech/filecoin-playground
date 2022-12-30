use fvm_ipld_blockstore::MemoryBlockstore;

use test_vm::{util::create_accounts, VM};
use fvm_shared::{econ::TokenAmount, address::Address};

use serde::{Deserialize};
use std::{fs};
use std::{fs::DirEntry, path::PathBuf};
use anyhow::{Result, anyhow};
use ethabi::Token;

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
    outputs: Vec<String>,
}

const TESTS_DIR_PATH: &str = "./tests/";

fn main() {
    // setup
    let store = MemoryBlockstore::new();
    let vm = VM::new_with_singletons(&store);
    let user_addr = create_accounts(&vm, 1, TokenAmount::from_whole(10_000))[0];

    let contract_test_json_paths: Vec<DirEntry> = fs::read_dir(TESTS_DIR_PATH)
        .expect(&format!("❌ failed to read dir: {}", TESTS_DIR_PATH))
        .filter_map(|dir_entry| dir_entry.ok())
        .collect();

    for dir_entry in contract_test_json_paths {
        let path = dir_entry.path();
        println!("\nRunning: {}", path.display());

        let result = run_contract_tests(&vm, user_addr, &path);

        if let Err(err) = result {
            println!("❌ Error for {}: {:?}", path.display(), err);
        }
    }
}

fn run_contract_tests(vm: &VM, user_addr: Address, path: &PathBuf) -> Result<()> {

    // load a test json
    let contract_tests_str_result = fs::read_to_string(path);
    if let Err(err) = contract_tests_str_result {
        return Err(anyhow!("Unable to read contract tests json file: {}: {:?}", path.display(), err));
    }

    let contract_tests_str = contract_tests_str_result.unwrap();
    let contract_tests_result: Result<ContractTests, serde_json::Error> = serde_json::from_str(&contract_tests_str);
    if let Err(err) = contract_tests_result {
        return Err(anyhow!("Unable deserialize contract tests json for: {}: {:?}", path.display(), err));
    }

    let contract_tests = contract_tests_result.unwrap();
    let test_runner = ContractTestRunner::new(&vm, &contract_tests.bytecode_file_path, &contract_tests.abi_file_path, user_addr)?;

    let mut failed_count: u32 = 0;
    let mut success_count: u32 = 0;

    // run all tests for this json
    for (i, contract_test) in contract_tests.tests.iter().enumerate() {
        if run_contract_test(&test_runner, user_addr, i, contract_test) {
            success_count += 1;
        }
        else {
            failed_count += 1;
        }
    }

    println!("{} succeeded, {} failed.", success_count, failed_count);

    Ok(())
}

fn run_contract_test(test_runner: &ContractTestRunner, user_addr: Address, index: usize, contract_test: &ContractTest) -> bool {
    let inputs: Vec<&str> = contract_test.inputs.iter().map(String::as_str).collect();
    let outputs_result = test_runner.invoke_function(user_addr, &contract_test.function_name_or_signature, &inputs);

    match outputs_result {
        Err(err) => {
            println!("\t❌ {}. {}({}) = Test error: {:?}", index, contract_test.function_name_or_signature, inputs.join(", "), err);
            false
        },
        Ok(outputs) => {
            // compare expected and produced
            match compare_contract_test_results(test_runner, contract_test, &outputs) {
                Err(compare_err) => {
                    println!("\t❌ {}. {}({}) = Test error: {:?}", index, contract_test.function_name_or_signature, inputs.join(", "), compare_err);
                    false
                },
                Ok(_) => {
                    println!("\t✅ {}. {}({}) = {:?}", index, contract_test.function_name_or_signature, inputs.join(", "), outputs);
                    true
                },
            }
        },
    }
}

fn compare_contract_test_results(test_runner: &ContractTestRunner, contract_test: &ContractTest, produced_outputs: &Vec<Token>) -> Result<()> {
    if contract_test.outputs.len() != produced_outputs.len() {
        return Err(anyhow!("Expected outputs length: {}, but produced outputs length: {}", contract_test.outputs.len(), produced_outputs.len()));
    }

    let expected_outputs_str: Vec<&str> = contract_test.outputs.iter().map(String::as_str).collect();
    let expected_outputs_result = test_runner.tokenize_abi_function_outputs(&contract_test.function_name_or_signature, &expected_outputs_str);
    if let Err(err) = expected_outputs_result {
        return Err(anyhow!("Failed to tokenize expected outputs: {:?}", err));
    }

    let expected_outputs = expected_outputs_result.unwrap();
    if expected_outputs.len() != produced_outputs.len() {
        return Err(anyhow!("Tokenized expected outputs length: {}, but produced outputs length: {}", expected_outputs.len(), produced_outputs.len()));
    }

    for (i, expected_output) in expected_outputs.iter().enumerate() {
        let produced_output = &produced_outputs[i];
        if expected_output != produced_output {
            return Err(anyhow!("Output of idx {} has expected value: {:?}, but produced value: {:?}", i, expected_output, produced_output));
        }
    }

    Ok(())
}