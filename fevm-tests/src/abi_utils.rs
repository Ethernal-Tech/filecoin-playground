use ethabi::{
	Contract, Function,
	token::{LenientTokenizer, Tokenizer, Token},
};

use std::fs;

pub fn abi_load_contract(file_path: &str) -> Option<Contract> {
    match fs::File::open(file_path) {
        Ok(abi_file) => {
            match Contract::load(abi_file) {
                Ok(contract_abi) => Some(contract_abi),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

pub fn abi_load_function(contract: &Contract, name_or_signature: &str) -> Option<Function> {
	let params_start = name_or_signature.find('(');

	match params_start {
		// It's a signature
		Some(params_start) => {
			let name = &name_or_signature[..params_start];

            let functions = contract.functions_by_name(name);
            match functions {
                Ok(fs) => fs.iter().find(|f| f.signature() == name_or_signature).cloned(),
                Err(_) => None,
            }
		}

		// It's a name
		None => {
			let functions = contract.functions_by_name(name_or_signature);
            match functions {
                Ok(fs) => {
                    match fs.len() {
                        1 => Some(fs[0].clone()),
                        _ => None,
                    }
                },
                Err(_) => None,
            }
			
		}
	}
}

pub fn abi_encode(function: &Function, inputs: &Vec<&str>) -> Option<Vec<u8>> {
	let params: Vec<_> = function.inputs.iter().map(|param| param.kind.clone()).zip(inputs.iter().map(|v| v as &str)).collect();
    let tokens: Result<Vec<Token>, ethabi::Error> = params.iter().map(|&(ref param, value)| LenientTokenizer::tokenize(param, value)).collect::<Result<_, _>>();

    match tokens {
        Ok(ts) => match function.encode_input(&ts) {
            Ok(encoded) => Some(encoded),
            Err(_) => None, 
        },
        Err(_) => None,
    }
}

pub fn abi_decode(function: &Function, to_decode: &Vec<u8>) -> Option<Vec<Token>> {
    match function.decode_output(to_decode) {
        Ok(decoded) => Some(decoded),
        Err(_) => None,
    }
}