//! @docs ARCHITECTURE:UI-Services
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[blueprint_service]` in tracing logs.

use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syn::{visit::Visit, Item};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub async fn scan_workspace(&self) -> Result<Blueprint> {
        if !self.root.exists() {
            return Err(anyhow::anyhow!(
                "Workspace root directory does not exist: {:?}",
                self.root
            ));
        }
        if !self.root.is_dir() {
            return Err(anyhow::anyhow!(
                "Workspace root path is not a directory: {:?}",
                self.root
            ));
        }

        let mut blueprint = Blueprint::default();
        let mut symbol_visitor = SymbolVisitor::default();
        let mut partial_errors = Vec::new();

        for entry in WalkDir::new(&self.root) {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    partial_errors.push(format!("WalkDir error: {}", err));
                    continue;
                }
            };
            if entry.path().extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            match std::fs::read_to_string(entry.path()) {
                Ok(content) => {
                    if let Ok(file) = syn::parse_file(&content) {
                        match entry.path().strip_prefix(&self.root) {
                            Ok(rel_path) => {
                                symbol_visitor.current_file =
                                    rel_path.to_string_lossy().to_string();
                                symbol_visitor.visit_file(&file);
                            }
                            Err(e) => {
                                partial_errors.push(format!(
                                    "Path prefix error for {:?}: {}",
                                    entry.path(),
                                    e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    partial_errors.push(format!("Failed to read file {:?}: {}", entry.path(), e));
                }
            }
        }

        blueprint.symbols = symbol_visitor.symbols;

        if !partial_errors.is_empty() {
            let error_msg = partial_errors.join("; ");
            tracing::warn!("Blueprint scan encountered partial errors: {}", error_msg);
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
                    line: 0, // In a real impl, we'd extract line info from spans
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

// Metadata: [blueprint_service]
