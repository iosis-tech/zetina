use crate::job::Job;

#[test]
fn job_serialize() {
    let random_job = Job::default();
    println!("{}", serde_json::to_string(&random_job).unwrap());
}
