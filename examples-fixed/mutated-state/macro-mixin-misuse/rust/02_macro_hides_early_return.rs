macro_rules! skip_blank {
    ($line:expr) => {
        if $line.trim().is_empty() {
            continue;
        }
    };
}

fn import(lines: &[&str], imported: &mut Vec<String>) {
    for line in lines {
        skip_blank!(line);
        imported.push(line.to_string());
    }
}

fn main() {
    let mut imported = Vec::new();
    import(&["alice", "", "bob", "carol"], &mut imported);
    println!("imported {:?}", imported);
}
