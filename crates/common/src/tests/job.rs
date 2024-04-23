use crate::job::Job;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn job_verify_signature(job in any::<Job>()) {
        assert!(job.verify_signature());
    }
}
