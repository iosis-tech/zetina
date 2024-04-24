use proptest::{
    arbitrary::{any, Arbitrary},
    prop_compose,
    strategy::{BoxedStrategy, Strategy},
};
use starknet::providers::sequencer::models::L1Address;

use crate::job::{Job, JobData};

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
            .prop_map(|(reward, num_of_steps, cairo_pie_compressed, secret_key)| {
                Job::from_job_data(
                    JobData {
                        reward,
                        num_of_steps,
                        cairo_pie_compressed,
                        registry_address: L1Address::random(),
                    },
                    libsecp256k1::SecretKey::parse(&secret_key).unwrap(),
                )
            })
            .boxed()
    }
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::arbitrary()
    }
}
