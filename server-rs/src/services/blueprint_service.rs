//! @docs ARCHITECTURE:Core:Intelligence:Blueprint
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / blueprint_service
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syn::{visit::Visit, Item};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: String, // "struct", "enum", "fn", "impl"
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintStatus {
    Complete,
    Degraded(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub symbols: Vec<Symbol>,
    pub dependencies: Vec<(String, String)>, // (SourceFile, TargetFile)
    pub status: BlueprintStatus,
}

impl Default for Blueprint {
    fn default() -> Self {
        Self {
            symbols: Vec::new(),
            dependencies: Vec::new(),
            status: BlueprintStatus::Complete,
        }
    }
}

pub struct BlueprintService {
    root: PathBuf,
}

impl BlueprintService {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Asynchronously scans the workspace AST on a blocking worker thread, filtering build directories.
    pub async fn scan_workspace(&self) -> Result<Blueprint> {
        let root = self.root.clone();
        if !root.exists() {
            return Err(anyhow::anyhow!(
                "Workspace root directory does not exist: {:?}",
                root
            ));
        }
        if !root.is_dir() {
            return Err(anyhow::anyhow!(
                "Workspace root path is not a directory: {:?}",
                root
            ));
        }

        tokio::task::spawn_blocking(move || Self::scan_workspace_blocking(&root))
            .await
            .map_err(|e| {
                anyhow::anyhow!("Blueprint background scan panicked or cancelled: {}", e)
            })?
    }

    fn scan_workspace_blocking(root: &std::path::Path) -> Result<Blueprint> {
        let mut blueprint = Blueprint::default();
        let mut symbol_visitor = SymbolVisitor::default();
        let mut partial_errors = Vec::new();

        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let name = entry.file_name().to_string_lossy();
                // Filter out build directories, version control metadata, and hidden files
                if entry.depth() > 0 && name.starts_with('.') {
                    return false;
                }
                name != "target" && name != "node_modules" && name != "dist" && name != "build"
            });

        for entry_res in walker {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    partial_errors.push(format!("WalkDir error: {}", err));
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            match std::fs::read_to_string(path) {
                Ok(content) => match syn::parse_file(&content) {
                    Ok(file) => match path.strip_prefix(root) {
                        Ok(rel_path) => {
                            symbol_visitor.current_file =
                                rel_path.to_string_lossy().replace('\\', "/");
                            symbol_visitor.visit_file(&file);
                        }
                        Err(e) => {
                            partial_errors.push(format!("Path prefix error for {:?}: {}", path, e));
                        }
                    },
                    Err(e) => {
                        let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
                        partial_errors.push(format!("Syntax parse error in {}: {}", rel, e));
                    }
                },
                Err(e) => {
                    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
                    partial_errors.push(format!("Failed to read file {}: {}", rel, e));
                }
            }
        }

        // Sort symbols deterministically for reliable caching and diffs
        symbol_visitor.symbols.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then_with(|| a.line.cmp(&b.line))
                .then_with(|| a.name.cmp(&b.name))
        });

        blueprint.symbols = symbol_visitor.symbols;

        if !partial_errors.is_empty() {
            let error_msg = partial_errors.join("; ");
            tracing::warn!(
                target: "blueprint_service",
                error_count = partial_errors.len(),
                "Blueprint scan encountered partial errors: {}",
                error_msg
            );
            blueprint.status = BlueprintStatus::Degraded(error_msg);
        }

        Ok(blueprint)
    }
}

#[derive(Default)]
struct SymbolVisitor {
    symbols: Vec<Symbol>,
    current_file: String,
}

impl<'ast> Visit<'ast> for SymbolVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Struct(s) => {
                self.symbols.push(Symbol {
                    name: s.ident.to_string(),
                    kind: "struct".to_string(),
                    file: self.current_file.clone(),
                    line: 0,
                });
            }
            Item::Enum(e) => {
                self.symbols.push(Symbol {
                    name: e.ident.to_string(),
                    kind: "enum".to_string(),
                    file: self.current_file.clone(),
                    line: 0,
                });
            }
            Item::Fn(f) => {
                self.symbols.push(Symbol {
                    name: f.sig.ident.to_string(),
                    kind: "fn".to_string(),
                    file: self.current_file.clone(),
                    line: 0,
                });
            }
            Item::Impl(im) => {
                if let syn::Type::Path(p) = &*im.self_ty {
                    if let Some(segment) = p.path.segments.last() {
                        self.symbols.push(Symbol {
                            name: format!("impl {}", segment.ident),
                            kind: "impl".to_string(),
                            file: self.current_file.clone(),
                            line: 0,
                        });
                    }
                }
            }
            _ => {}
        }
        syn::visit::visit_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_blueprint_scan_and_target_filter() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let target_dir = dir.path().join("target").join("debug");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&target_dir).unwrap();

        // Valid file in src/
        let mut f1 = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f1, "pub struct AppConfig; pub fn init_app() {{}}").unwrap();

        // File inside target/ (must be ignored)
        let mut f_target = File::create(target_dir.join("generated.rs")).unwrap();
        writeln!(f_target, "pub struct TargetGeneratedStruct;").unwrap();

        let service = BlueprintService::new(dir.path().to_path_buf());
        let blueprint = service.scan_workspace().await.unwrap();

        assert_eq!(blueprint.status, BlueprintStatus::Complete);
        assert_eq!(blueprint.symbols.len(), 2);
        let names: Vec<String> = blueprint.symbols.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"AppConfig".to_string()));
        assert!(names.contains(&"init_app".to_string()));
        assert!(!names.contains(&"TargetGeneratedStruct".to_string()));
    }

    #[tokio::test]
    async fn test_blueprint_degraded_status_on_syntax_error() {
        let dir = tempdir().unwrap();
        let mut f_bad = File::create(dir.path().join("broken.rs")).unwrap();
        writeln!(f_bad, "pub struct BadSyntax {{{{{{").unwrap();

        let service = BlueprintService::new(dir.path().to_path_buf());
        let blueprint = service.scan_workspace().await.unwrap();

        match blueprint.status {
            BlueprintStatus::Degraded(msg) => {
                assert!(msg.contains("Syntax parse error"));
            }
            BlueprintStatus::Complete => panic!("Must report Degraded on syntax parse error"),
        }
    }
}
