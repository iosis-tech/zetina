pub mod proofs;

use starknet::ContractAddress;
use snforge_std::{declare, ContractClassTrait};
use registry::{
    ISharpP2PRegistryDispatcher, ISharpP2PRegistryDispatcherTrait, SharpP2PRegistry::_get_metadata
};
use cairo_verifier::StarkProofWithSerde;

#[test]
fn test_metadata_deseralization() {
    let mut proof_serialized = proofs::fibonacci_proof::get().span();
    let proof = Serde::<StarkProofWithSerde>::deserialize(ref proof_serialized).unwrap();

    let metadata = _get_metadata(@proof.public_input);
    assert!(metadata.reward == 0x0000000000000000000000000000000000000000000000008dab53122f086e36);
    assert!(
        metadata.num_of_steps == 0x000000000000000000000000000000000000000000000000000000000000004b
    );
    assert!(
        metadata
            .executor == 0x000000000000000000000000000000000000000000000000000000000000000a
            .try_into()
            .unwrap()
    );
    assert!(
        metadata
            .delegator == 0x07778175a8dbc5317ff90701db5878fa8374fea1578bc14ef0451786d493ffe3
            .try_into()
            .unwrap()
    );
}
// #[test]
// fn test_increase_balance() {
//     let contract_address = deploy_contract("HelloStarknet");

//     let dispatcher = IHelloStarknetDispatcher { contract_address };

//     let balance_before = dispatcher.get_balance();
//     assert(balance_before == 0, 'Invalid balance');

//     dispatcher.increase_balance(42);

//     let balance_after = dispatcher.get_balance();
//     assert(balance_after == 42, 'Invalid balance');
// }

// #[test]
// #[feature("safe_dispatcher")]
// fn test_cannot_increase_balance_with_zero_value() {
//     let contract_address = deploy_contract("HelloStarknet");

//     let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

//     let balance_before = safe_dispatcher.get_balance().unwrap();
//     assert(balance_before == 0, 'Invalid balance');

//     match safe_dispatcher.increase_balance(0) {
//         Result::Ok(_) => core::panic_with_felt252('Should have panicked'),
//         Result::Err(panic_data) => {
//             assert(*panic_data.at(0) == 'Amount cannot be 0', *panic_data.at(0));
//         }
//     };
// }


