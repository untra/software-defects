use std::{env, fs, path::Path};

fn collect(path: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).expect("query directory") {
        let path = entry.expect("query entry").path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|e| e == "scm") {
            files.push(path);
        }
    }
}

fn main() {
    let root = Path::new("../../queries").canonicalize().expect("queries");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect(&root, &mut files);
    files.sort();
    let mut output = String::from("pub const QUERIES: &[(&str, &str)] = &[\n");
    for file in files {
        output.push_str(&format!(
            "({:?}, include_str!({:?})),\n",
            file.strip_prefix(&root).unwrap().to_str().unwrap(),
            file
        ));
    }
    output.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("queries.rs"),
        output,
    )
    .unwrap();
}
