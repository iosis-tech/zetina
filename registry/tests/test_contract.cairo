pub mod proofs;

use starknet::ContractAddress;
use snforge_std::{declare, ContractClassTrait};
use registry::{
    IZetinaRegistryDispatcher, IZetinaRegistryDispatcherTrait, ZetinaRegistry::get_metadata
};
use cairo_verifier::StarkProofWithSerde;

#[test]
fn test_metadata_deseralization() {
    let mut proof_serialized = proofs::fibonacci_proof::get().span();
    let proof = Serde::<StarkProofWithSerde>::deserialize(ref proof_serialized).unwrap();

    let metadata = get_metadata(@proof.public_input);
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
