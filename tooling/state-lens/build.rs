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
    files.retain(|file| {
        let relative = file.strip_prefix(&root).unwrap();
        let language = relative
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        env::var_os(format!("CARGO_FEATURE_{}", language.to_uppercase())).is_some()
    });
    files.sort();
    let mut metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("rules.json")).unwrap()).unwrap();
    metadata["rules"].as_array_mut().unwrap().retain(|rule| {
        env::var_os(format!(
            "CARGO_FEATURE_{}",
            rule["language"].as_str().unwrap().to_uppercase()
        ))
        .is_some()
    });
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("rules.json"),
        serde_json::to_string(&metadata).unwrap(),
    )
    .unwrap();
    let catalog_path = root.join("catalog.json");
    let mut catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&catalog_path).unwrap()).unwrap();
    let enabled: Vec<String> = catalog["languages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l.as_str())
        .filter(|l| env::var_os(format!("CARGO_FEATURE_{}", l.to_uppercase())).is_some())
        .map(str::to_owned)
        .collect();
    catalog["languages"] = serde_json::json!(enabled);
    for rule in catalog["rules"].as_array_mut().unwrap() {
        for mechanism in rule["mechanisms"].as_array_mut().unwrap() {
            let cells = mechanism["cells"].as_object_mut().unwrap();
            cells.retain(|language, _| enabled.contains(language));
        }
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("catalog.json"),
        serde_json::to_string(&catalog).unwrap(),
    )
    .unwrap();
    let mut tests = String::new();
    for rule in catalog["rules"].as_array().unwrap() {
        let defect = rule["defect"].as_str().unwrap();
        for mechanism in rule["mechanisms"].as_array().unwrap() {
            let id = mechanism["id"].as_str().unwrap();
            for (language, cell) in mechanism["cells"].as_object().unwrap() {
                let name =
                    |role: &str| format!("{defect}__{language}__{id}__{role}").replace('-', "_");
                if cell["status"] == "supported" || cell["role_fixtures"] == true {
                    for role in catalog["roles"].as_array().unwrap() {
                        let role = role.as_str().unwrap();
                        tests.push_str(&format!(
                            "#[test]\nfn {}() {{ role({defect:?}, {language:?}, {id:?}, {role:?}); }}\n",
                            name(role)
                        ));
                    }
                }
                tests.push_str(&format!(
                    "#[test]\nfn {}() {{ status({defect:?}, {language:?}, {id:?}); }}\n",
                    name("status")
                ));
            }
        }
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("matrix_tests.rs"),
        tests,
    )
    .unwrap();
    let mut output = String::from("pub const QUERIES: &[(&str, &str)] = &[\n");
    for file in files {
        let key = file
            .strip_prefix(&root)
            .unwrap()
            .components()
            .map(|part| part.as_os_str().to_str().unwrap())
            .collect::<Vec<_>>()
            .join("/");
        output.push_str(&format!("({key:?}, include_str!({file:?})),\n"));
    }
    output.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("queries.rs"),
        output,
    )
    .unwrap();
}
