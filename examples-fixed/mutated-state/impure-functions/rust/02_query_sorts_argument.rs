fn top_scorer(scores: &[(String, u32)]) -> String {
    scores
        .iter()
        .max_by_key(|score| score.1)
        .map(|score| score.0.clone())
        .unwrap_or_default()
}

fn main() {
    let roster_by_signup = vec![
        ("ana".to_string(), 3),
        ("ben".to_string(), 9),
        ("cy".to_string(), 5),
    ];
    let best = top_scorer(&roster_by_signup);
    println!("best {} / first signup {}", best, roster_by_signup[0].0);
}
