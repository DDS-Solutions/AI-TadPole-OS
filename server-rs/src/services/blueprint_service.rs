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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Blueprint {
    pub symbols: Vec<Symbol>,
    pub dependencies: Vec<(String, String)>, // (SourceFile, TargetFile)
}

pub struct BlueprintService {
    root: PathBuf,
}

impl BlueprintService {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn scan_workspace(&self) -> Result<Blueprint> {
        let mut blueprint = Blueprint::default();
        let mut symbol_visitor = SymbolVisitor::default();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let content = std::fs::read_to_string(entry.path())?;
            if let Ok(file) = syn::parse_file(&content) {
                symbol_visitor.current_file = entry
                    .path()
                    .strip_prefix(&self.root)?
                    .to_string_lossy()
                    .to_string();
                symbol_visitor.visit_file(&file);
            }
        }

        blueprint.symbols = symbol_visitor.symbols;
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
