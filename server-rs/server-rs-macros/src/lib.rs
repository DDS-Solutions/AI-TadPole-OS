//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **! Tadpole OS Procedural Macro Library**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[lib]` in tracing logs.

//! Tadpole OS Procedural Macro Library
//!
//! This crate provides the `#[agent_tool]` attribute macro, which acts as the
//! core synthesis engine for the modular tool registry. It automates the
//! generation of MCP-compliant JSON Schemas and boilerplate trait implementations
//! from standard Rust functions.
//!
//! @docs ARCHITECTURE:MacroRegistry
//! @docs STANDARDS:SovereignTooling

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, FnArg, Pat, Attribute};

/// Attribute macro that transforms a Rust function into an asynchronous `ToolHandler`.
///
/// This macro performs the following synthesis:
/// 1. **Doc Extraction**: Parses function doc comments and converts them into the `description` field.
/// 2. **Schema Generation**: Inspects function arguments to derive an `input_schema`.
/// 3. **Struct Synthesis**: Generates a unique `Handler` struct (e.g., `MyToolHandler`).
/// 4. **Trait Implementation**: Automatically implements `crate::agent::mcp::registry::ToolHandler`.
///
/// ### Example
/// ```rust
/// /// This tool performs a specialized calculation.
/// #[agent_tool]
/// pub async fn calculate(args: serde_json::Value, root: PathBuf) -> Result<McpResult> {
///    // ... implementation
/// }
/// ```
///
/// ### AI Assist Note
/// This macro eliminates manual schema maintenance. For AI assistants, the most
/// important aspect is that the internal `execute` call wraps the decorated function
/// exactly as defined.
#[proc_macro_attribute]
pub fn agent_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let docs = extract_docs(&input.attrs);
    let handler_name = quote::format_ident!("{}Handler", snake_to_camel(&fn_name.to_string()));

    // Generate JSON Schema from arguments
    let mut properties = quote! {};
    let mut required_names = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = pat_ident.ident.to_string();
                
                // Simple mapping for now
                let json_type = quote! { "string" }; 
                
                properties = quote! {
                    #properties
                    props.insert(#arg_name.to_string(), serde_json::json!({ "type": #json_type }));
                };
                
                required_names.push(arg_name);
            }
        }
    }

    let expanded = quote! {
        #input

        pub struct #handler_name;

        #[async_trait::async_trait]
        impl crate::agent::mcp::registry::ToolHandler for #handler_name {
            fn metadata(&self) -> crate::agent::mcp::McpToolHub {
                let mut props = std::collections::HashMap::<String, serde_json::Value>::new();
                #properties

                crate::agent::mcp::McpToolHub {
                    name: stringify!(#fn_name).to_string(),
                    description: #docs.to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": props,
                        "required": vec![ #(#required_names),* ]
                    }),
                    source: "system".to_string(),
                    stats: crate::agent::mcp::McpToolStats::default(),
                    category: "ai".to_string(),
                }
            }

            async fn execute(
                &self, 
                args: serde_json::Value, 
                workspace_root: std::path::PathBuf
            ) -> anyhow::Result<crate::agent::mcp::McpResult> {
                #fn_name(args, workspace_root).await
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extracts documentation comments from a list of attributes.
///
/// Concatenates multiple `#[doc = "..."]` attributes into a single string,
/// trimming whitespace and joining with spaces.
///
/// ### AI Assist Note
/// This is used to automatically generate the `description` field for the 
/// MCP tool metadata.
fn extract_docs(attrs: &[Attribute]) -> String {
    let mut docs = String::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        docs.push_str(&lit_str.value().trim());
                        docs.push(' ');
                    }
                }
            }
        }
    }
    docs.trim().to_string()
}

/// Converts a snake_case string (function name) to CamelCase (Handler struct name).
///
/// Example: `list_file_symbols` -> `ListFileSymbolsHandler`
fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

// Metadata: [lib]
