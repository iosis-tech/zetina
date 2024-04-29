use crate::job::{Job, JobData};

use proptest::{
    arbitrary::{any, Arbitrary},
    prop_compose,
    strategy::{BoxedStrategy, Strategy},
};

use starknet::signers::SigningKey;
use starknet_crypto::FieldElement;

// Disabled coz i cannot generate random valid cairo_pie_compressed fields yet

// This generates a random Job object for testing purposes.
// prop_compose! {
//     fn arb_state()(
//         reward in any::<u64>(),
//         num_of_steps in any::<u32>(),
//         cairo_pie_compressed in any::<Vec<u8>>(),
//         secret_key in any::<[u8; 32]>()
//     ) -> (u32, u32, Vec<u8>, [u8; 32]) {
//         (reward, num_of_steps, cairo_pie_compressed, secret_key)
//     }
// }

// impl Arbitrary for Job {
//     type Parameters = ();
//     type Strategy = BoxedStrategy<Self>;
//     fn arbitrary() -> Self::Strategy {
//         let abs_state = arb_state();
//         abs_state
//             .prop_map(|(reward, num_of_steps, cairo_pie_compressed, _)| {
//                 Job::try_from_job_data(
//                     JobData {
//                         reward,
//                         num_of_steps,
//                         cairo_pie_compressed,
//                         registry_address: FieldElement::ZERO,
//                     },
//                     &SigningKey::from_random(),
//                 )
//             })
//             .boxed()
//     }
//     fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
//         Self::arbitrary()
//     }
// }
