fn top_scorer(scores: &mut Vec<(String, u32)>) -> String {
    scores.sort_by(|a, b| b.1.cmp(&a.1));
    scores[0].0.clone()
}

fn main() {
    let mut roster_by_signup = vec![
        ("ana".to_string(), 3),
        ("ben".to_string(), 9),
        ("cy".to_string(), 5),
    ];
    let best = top_scorer(&mut roster_by_signup);
    println!("best {} / first signup {}", best, roster_by_signup[0].0);
}
