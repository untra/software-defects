const GUESTS: [&str; 4] = ["Ari", "Bo", "Cy", "Dee"];
fn roster(names: &[&str]) -> usize {
    let mut accepted = Vec::new();
    let mut comparisons = 0;
    for name in names {
        let mut present = false;
        for earlier in &accepted {
            comparisons += 1;
            if earlier == name { present = true; }
        }
        if !present { accepted.push(*name); }
    }
    comparisons
}
fn main() { println!("comparisons {}", roster(&GUESTS)); }
