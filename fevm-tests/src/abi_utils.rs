use ethabi::{
	Contract, Function,
	token::{LenientTokenizer, Tokenizer, Token},
};

use std::fs;
use anyhow::{Result, anyhow};

pub fn abi_load_contract(file_path: &str) -> Result<Contract> {
    let abi_file_result = fs::File::open(file_path);
    if let Err(_err) = abi_file_result {
        return Err(anyhow!("failed to open file at: {}", file_path));
    }

    let contract_abi_result = Contract::load(abi_file_result.unwrap());
    if let Err(_err) = contract_abi_result {
        return Err(anyhow!("failed to load abi contract at: {}", file_path));
    }
    
    Ok(contract_abi_result.unwrap())
}

pub fn abi_load_function(contract: &Contract, name_or_signature: &str) -> Result<Function> {
	let params_start = name_or_signature.find('(');

	match params_start {
		// It's a signature
		Some(params_start) => {
			let name = &name_or_signature[..params_start];
            
            let functions = contract.functions_by_name(name);
            if let Err(_err) = functions {
                return Err(anyhow!("invalid function signature `{}`", name_or_signature));
            }

			functions
                .unwrap()
				.iter()
				.find(|f| f.signature() == name_or_signature)
				.cloned()
				.ok_or_else(|| anyhow!("invalid function signature `{}`", name_or_signature))
		}

		// It's a name
		None => {
			let functions_result = contract.functions_by_name(name_or_signature);
            if let Err(_err) = functions_result {
                return Err(anyhow!("invalid function name `{}`", name_or_signature));
            }
            
            let functions = functions_result.unwrap();

            match functions.len() {
                0 => Err(anyhow!("No functions found for name `{}`", name_or_signature)),
                1 => Ok(functions[0].clone()),
                _ => Err(anyhow!("More than one function found for name `{}`, try providing the full signature", name_or_signature)),
            }			
		}
	}
}

pub fn abi_encode(function: &Function, inputs: &Vec<&str>) -> Result<Vec<u8>> {
    let tokens = abi_tokenize_function_inputs(function, inputs)?;
    let encoded_result = function.encode_input(&tokens);
    if let Err(_err) = encoded_result {
        return Err(anyhow!("failed to abi encode"));
    }
    
    Ok(encoded_result.unwrap())
}

pub fn abi_tokenize_function_inputs(function: &Function, inputs: &Vec<&str>) -> Result<Vec<Token>> {
    let params: Vec<_> = function.inputs
        .iter()
        .map(|param| param.kind.clone())
        .zip(inputs.iter().map(|v| v as &str))
        .collect();

    let tokens_result: Result<Vec<Token>, ethabi::Error> = params
        .iter()
        .map(|&(ref param, value)| LenientTokenizer::tokenize(param, value))
        .collect::<Result<_, _>>();
    
    if let Err(_err) = tokens_result {
        return Err(anyhow!("failed to tokenize function inputs"));
    }

    Ok(tokens_result.unwrap())
}

pub fn abi_tokenize_function_outputs(function: &Function, outputs: &Vec<&str>) -> Result<Vec<Token>> {
    let params: Vec<_> = function.outputs
        .iter()
        .map(|param| param.kind.clone())
        .zip(outputs.iter().map(|v| v as &str))
        .collect();

    let tokens_result: Result<Vec<Token>, ethabi::Error> = params
        .iter()
        .map(|&(ref param, value)| LenientTokenizer::tokenize(param, value))
        .collect::<Result<_, _>>();
    
    if let Err(_err) = tokens_result {
        return Err(anyhow!("failed to tokenize function outputs"));
    }

    Ok(tokens_result.unwrap())
}

pub fn abi_decode(function: &Function, to_decode: &Vec<u8>) -> Result<Vec<Token>> {
    let decoded_result = function.decode_output(to_decode);
    if let Err(_err) = decoded_result {
        return Err(anyhow!("failed to abi decode"));
    }

    Ok(decoded_result.unwrap())
}