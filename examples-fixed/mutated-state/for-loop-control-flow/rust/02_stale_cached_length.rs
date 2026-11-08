const RETRY_JOB: &str = "job-a";

fn main() {
    let mut queue = vec!["job-a".to_string(), "job-b".to_string()];
    let mut i = 0;
    while i < queue.len() {
        let job = queue[i].clone();
        if job == RETRY_JOB {
            queue.push(format!("{}-retry", job));
        }
        println!("processed {}", job);
        i += 1;
    }
    println!("unprocessed: {:?}", &queue[i..]);
}
