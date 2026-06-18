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
/// ```rust,ignore
/// /// This tool performs a specialized calculation.
/// #[agent_tool]
/// pub async fn calculate(amount: i32, workspace_root: PathBuf) -> Result<McpResult, AppError> {
///    // ... implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn agent_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let docs = extract_docs(&input.attrs);
    let handler_name = quote::format_ident!("{}Handler", snake_to_camel(&fn_name.to_string()));

    // Generate JSON Schema and deserialization logic from arguments
    let mut properties = quote! {};
    let mut required_names = Vec::new();
    let mut deserialize_statements = Vec::new();
    let mut call_args = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                let arg_name_str = arg_name.to_string();
                let arg_ty = &pat_type.ty;

                if arg_name_str == "workspace_root" || arg_name_str == "_workspace_root" {
                    // System argument (workspace root) is injected directly from MCP execution context
                    call_args.push(quote! { workspace_root });
                } else {
                    // Tool parameter
                    let (json_type, is_optional) = map_type_to_schema(arg_ty);
                    let name_for_props = arg_name_str.clone();
                    let name_for_required = arg_name_str.clone();
                    let name_for_get = arg_name_str.clone();
                    let name_for_err = arg_name_str.clone();

                    properties = quote! {
                        #properties
                        props.insert(#name_for_props.to_string(), serde_json::json!({ "type": #json_type }));
                    };

                    if !is_optional {
                        required_names.push(name_for_required);
                    }

                    deserialize_statements.push(quote! {
                        let #arg_name: #arg_ty = serde_json::from_value(
                            args.get(#name_for_get).cloned().unwrap_or(serde_json::Value::Null)
                        ).map_err(|e| crate::error::AppError::BadRequest(
                            format!("Invalid parameter '{}': {}", #name_for_err, e)
                        ))?;
                    });

                    call_args.push(quote! { #arg_name });
                }
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

                let required: Vec<&str> = vec![ #(#required_names),* ];

                crate::agent::mcp::McpToolHub {
                    name: stringify!(#fn_name).to_string(),
                    description: #docs.to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": props,
                        "required": required
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
            ) -> Result<crate::agent::mcp::McpResult, crate::error::AppError> {
                #(#deserialize_statements)*
                #fn_name(#(#call_args),*).await
            }
        }
    };

    TokenStream::from(expanded)
}

/// Maps Rust types to JSON Schema types and returns whether they are optional.
fn map_type_to_schema(ty: &syn::Type) -> (&'static str, bool) {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident_str = segment.ident.to_string();
            if ident_str == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        let (inner_type_str, _) = map_type_to_schema(inner_ty);
                        return (inner_type_str, true);
                    }
                }
                return ("string", true);
            }
            if ident_str == "Vec" {
                return ("array", false);
            }
            let json_type = match ident_str.as_str() {
                "String" | "str" => "string",
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => "integer",
                "f32" | "f64" => "number",
                "bool" => "boolean",
                "Value" | "Map" => "object",
                _ => "string",
            };
            return (json_type, false);
        }
    }
    ("string", false)
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

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("hello_world"), "HelloWorld");
        assert_eq!(snake_to_camel("test_fn_name"), "TestFnName");
        assert_eq!(snake_to_camel("alreadyCamel"), "AlreadyCamel");
        assert_eq!(snake_to_camel("multiple__underscores"), "MultipleUnderscores");
        assert_eq!(snake_to_camel("_leading"), "Leading");
        assert_eq!(snake_to_camel("trailing_"), "Trailing");
    }

    #[test]
    fn test_extract_docs() {
        use syn::{parse_quote, Attribute};

        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[doc = " Line one. "]),
            parse_quote!(#[doc = "Line two."]),
            parse_quote!(#[other_attr]),
        ];

        let docs = extract_docs(&attrs);
        assert_eq!(docs, "Line one. Line two.");
    }
}

// Metadata: [lib]
