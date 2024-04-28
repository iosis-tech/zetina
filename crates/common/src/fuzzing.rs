use crate::job::{Job, JobData};

use proptest::{
    arbitrary::{any, Arbitrary},
    prop_compose,
    strategy::{BoxedStrategy, Strategy},
};

use starknet_crypto::FieldElement;

// This generates a random Job object for testing purposes.
prop_compose! {
    fn arb_state()(
        reward in any::<u32>(),
        num_of_steps in any::<u32>(),
        cairo_pie_compressed in any::<Vec<u8>>(),
        secret_key in any::<[u8; 32]>()
    ) -> (u32, u32, Vec<u8>, [u8; 32]) {
        (reward, num_of_steps, cairo_pie_compressed, secret_key)
    }
}

impl Arbitrary for Job {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary() -> Self::Strategy {
        let abs_state = arb_state();
        abs_state
            .prop_map(|(reward, num_of_steps, cairo_pie_compressed, _)| {
                Job::from_job_data(
                    JobData {
                        reward,
                        num_of_steps,
                        cairo_pie_compressed,
                        registry_address: FieldElement::ZERO,
                    },
                    &libp2p::identity::ecdsa::Keypair::generate(),
                )
            })
            .boxed()
    }
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::arbitrary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn job_verify_signature(job in any::<Job>()) {
            assert!(job.verify_signature());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn job_serialization(job in any::<Job>()) {
            let serialized_job = serde_json::to_string(&job).unwrap();
            let deserialized_job: Job = serde_json::from_str(&serialized_job).unwrap();
            assert_eq!(job, deserialized_job)
        }
    }
}
