use fvm_shared::{econ::TokenAmount};
use std::cell::{RefCell};
use fil_actors_runtime::{runtime::{Policy}, EAM_ACTOR_ADDR};
use num_traits::Zero;
use test_vm::{VM, TopCtx, InternalMessage, InvocationCtx, util::create_accounts};

pub fn mock_contract_runtime<'a,'b>(vm: &'b VM<'b>, policy: &'a Policy, bytecode: Vec<u8>) -> InvocationCtx<'a, 'b> {
    let from = create_accounts(&vm, 1, TokenAmount::from_whole(10_000))[0];
    
    let create_result = vm.apply_message(
        from,
        EAM_ACTOR_ADDR,
        TokenAmount::zero(),
        fil_actor_eam::Method::Create2 as u64,
        Some(fil_actor_eam::Create2Params { initcode: bytecode, salt: [0u8; 32] }),
    )
    .unwrap();

    if !create_result.code.is_success() {
        panic!("create failed");
    }

    let create_return: fil_actor_eam::Create2Return = create_result.ret.deserialize().expect("failed to decode results");
    
    let from_id = vm.normalize_address(&from).unwrap();
    let mut a = vm.get_actor(from_id).unwrap();
    let call_seq = a.call_seq_num;
    a.call_seq_num = call_seq + 1;
    vm.set_actor(from_id, a);
    // let prior_root = vm.checkpoint();

    let top = TopCtx {
        originator_stable_addr: from,
        originator_call_seq: call_seq,
        new_actor_addr_count: RefCell::new(0),
        circ_supply: TokenAmount::from_whole(1_000_000_000),
    };
    let msg = InternalMessage {
        from: from_id,
        to: create_return.robust_address,
        value: TokenAmount::zero(),
        method: fil_actor_evm::Method::InvokeContract as u64,
        params: std::option::Option::None//params.map(|p| IpldBlock::serialize_cbor(&p).unwrap().unwrap()),
    };
    let new_ctx = InvocationCtx {
        v: &vm,
        top,
        msg,
        allow_side_effects: true,
        caller_validated: false,
        policy: policy,
        subinvocations: RefCell::new(vec![]),
        actor_exit: RefCell::new(None),
        read_only: false,
    };
    new_ctx
}
