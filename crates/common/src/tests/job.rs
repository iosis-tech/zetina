use crate::job::Job;
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
