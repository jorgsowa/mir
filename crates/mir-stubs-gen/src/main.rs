/// mir-stubs-gen — transforms PHP stub files into Rust source for mir-analyzer.
///
/// Usage:
///   cargo run -p mir-stubs-gen              # regenerate all extensions
///   cargo run -p mir-stubs-gen -- curl      # regenerate one extension
///
/// Input:  stubs/{ext}/stub.toml + stubs/{ext}/*.php
/// Output: crates/mir-analyzer/src/generated/stubs_{ext}.rs
///         crates/mir-analyzer/src/generated/mod.rs  (updated)
use std::fmt::Write as FmtWrite;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mir_codebase::{Codebase, StubSlice};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Manifest format — stubs/{ext}/stub.toml
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct StubManifest {
    extension: ExtensionMeta,
}

#[derive(Deserialize)]
struct ExtensionMeta {
    #[allow(dead_code)]
    name: String,
    version: String,
    #[serde(rename = "php-min")]
    php_min: String,
    #[serde(rename = "php-max", default)]
    _php_max: Option<String>,
    #[serde(default)]
    _composer: Option<String>,
    #[serde(default)]
    _description: Option<String>,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let filter: Option<&str> = args.first().map(|s| s.as_str());

    let workspace_root = find_workspace_root().expect("could not locate workspace root");
    let stubs_dir = workspace_root.join("stubs");
    let out_dir = workspace_root
        .join("crates")
        .join("mir-analyzer")
        .join("src")
        .join("generated");

    std::fs::create_dir_all(&out_dir).unwrap();

    if !stubs_dir.is_dir() {
        eprintln!("No stubs/ directory found at {}", workspace_root.display());
        std::process::exit(1);
    }

    let mut ext_dirs: Vec<PathBuf> = std::fs::read_dir(&stubs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    ext_dirs.sort();

    let mut generated_modules: Vec<String> = Vec::new();

    for ext_dir in &ext_dirs {
        let ext_name = ext_dir.file_name().unwrap().to_string_lossy().to_string();

        if let Some(filter) = filter {
            if ext_name != filter {
                continue;
            }
        }

        let manifest_path = ext_dir.join("stub.toml");
        if !manifest_path.exists() {
            eprintln!("skipping {ext_name}: no stub.toml");
            continue;
        }

        let manifest_src = std::fs::read_to_string(&manifest_path).unwrap();
        let manifest: StubManifest = toml::from_str(&manifest_src).unwrap_or_else(|e| {
            panic!("failed to parse {}: {e}", manifest_path.display());
        });
        let meta = &manifest.extension;

        println!("generating stubs_{ext_name} (version {})", meta.version);

        let input_hash = hash_input_tree(ext_dir);

        let slice = collect_stubs(ext_dir, &workspace_root);

        let encoded: Vec<u8> = bincode::serde::encode_to_vec(&slice, bincode::config::standard())
            .expect("bincode encode failed");

        let module_name = format!("stubs_{}", ext_name.replace('-', "_"));
        let out_path = out_dir.join(format!("{module_name}.rs"));

        write_generated_file(
            &out_path,
            &module_name,
            &ext_name,
            &meta.version,
            &meta.php_min,
            &input_hash,
            &encoded,
        );

        generated_modules.push(module_name);
    }

    // Update mod.rs to list all generated modules.
    if filter.is_none() {
        write_mod_rs(&out_dir, &generated_modules);
    } else if let Some(name) = filter {
        // Single-extension mode: merge with existing mod.rs.
        merge_mod_rs(&out_dir, &format!("stubs_{}", name.replace('-', "_")));
    }

    println!("done.");
}

// ---------------------------------------------------------------------------
// PHP stub collection
// ---------------------------------------------------------------------------

fn collect_stubs(ext_dir: &Path, workspace_root: &Path) -> StubSlice {
    let codebase = Codebase::new();

    let mut php_files: Vec<PathBuf> = collect_php_files(ext_dir);
    php_files.sort();

    for php_path in &php_files {
        let content = std::fs::read_to_string(php_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", php_path.display()));

        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, &content);
        // Use a workspace-relative filename so generated stubs are byte-identical
        // regardless of where the repo is checked out.
        let rel = php_path.strip_prefix(workspace_root).unwrap_or(php_path);
        let filename: Arc<str> = Arc::from(rel.to_string_lossy().as_ref());
        let collector = mir_analyzer::collector::DefinitionCollector::new(
            &codebase,
            filename,
            &content,
            &result.source_map,
        );
        let _ = collector.collect(&result.program);
    }

    // Strip source locations so generated files are portable across machines.
    // Sort each collection by name so DashMap iteration order doesn't leak into
    // the serialized output — without this, consecutive regen runs produce
    // different bytes and `verify-stub-hashes.sh` can't gate on byte-identity.
    let mut classes: Vec<_> = codebase
        .classes
        .iter()
        .map(|e| strip_class_location(e.value().clone()))
        .collect();
    classes.sort_by(|a, b| a.fqcn.cmp(&b.fqcn));

    let mut interfaces: Vec<_> = codebase
        .interfaces
        .iter()
        .map(|e| strip_interface_location(e.value().clone()))
        .collect();
    interfaces.sort_by(|a, b| a.fqcn.cmp(&b.fqcn));

    let mut traits: Vec<_> = codebase
        .traits
        .iter()
        .map(|e| strip_trait_location(e.value().clone()))
        .collect();
    traits.sort_by(|a, b| a.fqcn.cmp(&b.fqcn));

    let mut enums: Vec<_> = codebase
        .enums
        .iter()
        .map(|e| strip_enum_location(e.value().clone()))
        .collect();
    enums.sort_by(|a, b| a.fqcn.cmp(&b.fqcn));

    let mut functions: Vec<_> = codebase
        .functions
        .iter()
        .map(|e| strip_fn_location(e.value().clone()))
        .collect();
    functions.sort_by(|a, b| a.fqn.cmp(&b.fqn));

    let mut constants: Vec<_> = codebase
        .constants
        .iter()
        .map(|e| (e.key().clone(), e.value().clone()))
        .collect();
    constants.sort_by(|a, b| a.0.cmp(&b.0));

    StubSlice {
        classes,
        interfaces,
        traits,
        enums,
        functions,
        constants,
    }
}

// ---------------------------------------------------------------------------
// Location stripping — makes generated files portable across machines
// ---------------------------------------------------------------------------

fn strip_fn_location(mut f: mir_codebase::FunctionStorage) -> mir_codebase::FunctionStorage {
    f.location = None;
    f
}

fn strip_method_location(
    mut m: mir_codebase::storage::MethodStorage,
) -> mir_codebase::storage::MethodStorage {
    m.location = None;
    m
}

fn strip_class_location(mut cls: mir_codebase::ClassStorage) -> mir_codebase::ClassStorage {
    cls.location = None;
    cls.own_methods = cls
        .own_methods
        .into_iter()
        .map(|(k, m)| {
            let m = strip_method_location((*m).clone());
            (k, std::sync::Arc::new(m))
        })
        .collect();
    cls
}

fn strip_interface_location(
    mut iface: mir_codebase::InterfaceStorage,
) -> mir_codebase::InterfaceStorage {
    iface.location = None;
    iface.own_methods = iface
        .own_methods
        .into_iter()
        .map(|(k, m)| {
            let m = strip_method_location((*m).clone());
            (k, std::sync::Arc::new(m))
        })
        .collect();
    iface
}

fn strip_trait_location(mut tr: mir_codebase::TraitStorage) -> mir_codebase::TraitStorage {
    tr.location = None;
    tr.own_methods = tr
        .own_methods
        .into_iter()
        .map(|(k, m)| {
            let m = strip_method_location((*m).clone());
            (k, std::sync::Arc::new(m))
        })
        .collect();
    tr
}

fn strip_enum_location(mut en: mir_codebase::EnumStorage) -> mir_codebase::EnumStorage {
    en.location = None;
    en.own_methods = en
        .own_methods
        .into_iter()
        .map(|(k, m)| {
            let m = strip_method_location((*m).clone());
            (k, std::sync::Arc::new(m))
        })
        .collect();
    en
}

fn collect_php_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_php_files(&path));
            } else if path.extension().is_some_and(|e| e == "php") {
                result.push(path);
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Input hash — blake3 over sorted (relative-path, content) pairs for stubs/{ext}/
// ---------------------------------------------------------------------------

/// Deterministic hash over stubs/{ext}/: for each file in sorted relative-path
/// order, feeds `relpath \0 content \0` into blake3. The format is trivially
/// reproducible from a shell script so CI can verify without compiling.
fn hash_input_tree(ext_dir: &Path) -> String {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_all_files(ext_dir, &mut files);
    files.sort();

    let mut hasher = blake3::Hasher::new();
    for path in &files {
        let rel = path.strip_prefix(ext_dir).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        let content = std::fs::read(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        hasher.update(rel_str.as_bytes());
        hasher.update(&[0u8]);
        hasher.update(&content);
        hasher.update(&[0u8]);
    }
    hasher.finalize().to_hex().to_string()
}

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_all_files(&path, out);
            } else {
                out.push(path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generated file writer
// ---------------------------------------------------------------------------

fn write_generated_file(
    out_path: &Path,
    module_name: &str,
    ext_name: &str,
    version: &str,
    php_min: &str,
    input_hash: &str,
    encoded: &[u8],
) {
    let mut code = String::new();

    writeln!(
        code,
        "// Generated from stubs/{ext_name}/ — version {version} | php >= {php_min}"
    )
    .unwrap();
    writeln!(
        code,
        "// Run `cargo run -p mir-stubs-gen -- {ext_name}` to regenerate"
    )
    .unwrap();
    writeln!(code, "// DO NOT EDIT DIRECTLY").unwrap();
    writeln!(code, "// input-hash: blake3:{input_hash}").unwrap();
    writeln!(code).unwrap();

    // Embed the bincode-encoded StubSlice as a byte array.
    writeln!(code, "static DATA: &[u8] = &[").unwrap();
    let mut line = String::from("   ");
    for (i, byte) in encoded.iter().enumerate() {
        write!(line, " {byte:#04x},").unwrap();
        if (i + 1) % 16 == 0 {
            writeln!(code, "{line}").unwrap();
            line = String::from("   ");
        }
    }
    if !line.trim().is_empty() {
        writeln!(code, "{line}").unwrap();
    }
    writeln!(code, "];").unwrap();
    writeln!(code).unwrap();

    writeln!(
        code,
        "pub(crate) fn register(codebase: &mir_codebase::Codebase) {{"
    )
    .unwrap();
    writeln!(code, "    let (slice, _): (mir_codebase::StubSlice, _) =").unwrap();
    writeln!(
        code,
        "        bincode::serde::decode_from_slice(DATA, bincode::config::standard())"
    )
    .unwrap();
    writeln!(
        code,
        "            .expect(\"corrupt {module_name} stub data\");"
    )
    .unwrap();
    writeln!(code, "    codebase.inject_stub_slice(slice);").unwrap();
    writeln!(code, "}}").unwrap();

    std::fs::write(out_path, code).unwrap();
    println!("  wrote {}", out_path.display());
}

// ---------------------------------------------------------------------------
// mod.rs management
// ---------------------------------------------------------------------------

fn write_mod_rs(out_dir: &Path, modules: &[String]) {
    let mut sorted = modules.to_vec();
    sorted.sort();
    let path = out_dir.join("mod.rs");
    std::fs::write(&path, render_mod_rs(&sorted)).unwrap();
    println!("  wrote {}", path.display());
}

fn merge_mod_rs(out_dir: &Path, module_name: &str) {
    let path = out_dir.join("mod.rs");
    let mut modules: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter_map(|l| {
                l.strip_prefix("pub(crate) mod ")
                    .and_then(|s| s.strip_suffix(';'))
                    .map(str::to_string)
            })
            .collect()
    } else {
        Vec::new()
    };
    if !modules.iter().any(|m| m == module_name) {
        modules.push(module_name.to_string());
    }
    modules.sort();
    modules.dedup();
    std::fs::write(&path, render_mod_rs(&modules)).unwrap();
    println!("  updated {}", path.display());
}

fn render_mod_rs(modules: &[String]) -> String {
    let mut code = String::from("// Auto-generated by mir-stubs-gen — do not edit directly.\n\n");
    for module in modules {
        writeln!(code, "pub(crate) mod {module};").unwrap();
    }
    writeln!(code).unwrap();
    writeln!(
        code,
        "/// Registers every generated stub module into `codebase`."
    )
    .unwrap();
    writeln!(
        code,
        "pub(crate) fn register_all(codebase: &mir_codebase::Codebase) {{"
    )
    .unwrap();
    for module in modules {
        writeln!(code, "    {module}::register(codebase);").unwrap();
    }
    writeln!(code, "}}").unwrap();
    code
}

// ---------------------------------------------------------------------------
// Workspace root detection
// ---------------------------------------------------------------------------

fn find_workspace_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml).ok()?;
            if content.contains("[workspace]") {
                return Some(dir);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}
