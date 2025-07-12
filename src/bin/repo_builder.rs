use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path};

use toml::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_dir = env::args().nth(1).expect("Usage: repo_builder <REPO_DIR>");
    let repo_path = Path::new(&repo_dir);
    let repo_toml_path = repo_path.join("repo.toml");

    let mut packages: BTreeMap<String, String> = BTreeMap::new();

    // 1. Read existing repo.toml if it exists
    if repo_toml_path.exists() {
        let mut file = File::open(&repo_toml_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let parsed: Value = toml::from_str(&contents)?;
        if let Some(pkg_table) = parsed.get("packages").and_then(|v| v.as_table()) {
            for (k, v) in pkg_table {
                if let Some(s) = v.as_str() {
                    packages.insert(k.clone(), format!("\"{}\"", s));
                } else {
                    packages.insert(k.clone(), v.to_string());
                }
            }
        }
    }

    // 2. Scan all .toml files and update packages map
    for entry in fs::read_dir(&repo_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        if path.file_stem().and_then(|s| s.to_str()) == Some("repo") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let parsed: Value = toml::from_str(&content)?;

        if let Some(version_val) = parsed.get("version") {
            let version_str = version_val.to_string(); // includes quotes
            let package_name = path.file_stem().unwrap().to_string_lossy().to_string();
            packages.insert(package_name, version_str);
        } else {
            eprintln!("Warning: no [version] found in {:?}", path);
        }
    }

    // 3. Write updated repo.toml
    // FIXME: Use proper TOML serializer
    let mut output = String::from("[packages]\n");
    for (name, version) in &packages {
        output.push_str(&format!("{name} = {version}\n"));
    }

    let mut output_file = File::create(&repo_toml_path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}
