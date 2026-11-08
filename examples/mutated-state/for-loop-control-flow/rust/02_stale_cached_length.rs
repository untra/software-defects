const RETRY_JOB: &str = "job-a";

fn main() {
    let mut queue = vec!["job-a".to_string(), "job-b".to_string()];
    let pending = queue.len();
    for i in 0..pending {
        let job = queue[i].clone();
        if job == RETRY_JOB {
            queue.push(format!("{}-retry", job));
        }
        println!("processed {}", job);
    }
    println!("unprocessed: {:?}", &queue[pending..]);
}
