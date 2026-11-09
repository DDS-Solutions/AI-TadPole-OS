//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Graph Query CLI / route_parity
//! - **Primary Entrypoints**: `run_route_parity`, `extract_routes_from_router_ast`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic AST route extraction and OpenAPI YAML parity validation.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `GraphQueryError::Validation`, `GraphQueryError::Io`
//! - **Telemetry Targets**: none declared

use crate::GraphQueryError;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprLit, ExprMethodCall, ItemFn, Lit};

const HTTP_VERBS: [&str; 5] = ["get", "post", "put", "delete", "patch"];
const DIRECT_V1_BUILDERS: [&str; 5] = [
    "build_protected_v1_routes",
    "build_engine_public_routes",
    "build_engine_protected_routes",
    "build_remote_public_routes",
    "build_remote_paired_routes",
];

#[derive(Debug, Clone, Serialize)]
pub struct ExtractedRoute {
    pub path: String,
    pub method: String,
    pub handler: String,
}

struct FnRouteVisitor {
    routes: Vec<ExtractedRoute>,
    nests: Vec<(String, String)>,
}

fn normalize_route_path(path: &str) -> String {
    let mut out = String::new();
    for seg in path.split('/') {
        if seg.is_empty() {
            continue;
        }
        out.push('/');
        if seg.starts_with(':') || (seg.starts_with('{') && seg.ends_with('}')) {
            out.push_str("{PARAM}");
        } else {
            out.push_str(seg);
        }
    }
    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

fn quote_like_expr(expr: &Expr) -> String {
    let Expr::Path(p) = expr else {
        return "handler".to_string();
    };
    p.path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn push_verb_if_valid(
    ident: &str,
    args: &syn::punctuated::Punctuated<Expr, syn::Token![,]>,
    out: &mut Vec<(String, String)>,
) {
    if HTTP_VERBS.contains(&ident) {
        let handler = args.first().map(quote_like_expr).unwrap_or_default();
        out.push((ident.to_uppercase(), handler));
    }
}

fn collect_http_methods(expr: &Expr, out: &mut Vec<(String, String)>) {
    if let Expr::Call(ExprCall { func, args, .. }) = expr {
        let Expr::Path(p) = &**func else { return };
        if let Some(seg) = p.path.segments.last() {
            push_verb_if_valid(&seg.ident.to_string(), args, out);
        }
    } else if let Expr::MethodCall(ExprMethodCall {
        receiver,
        method,
        args,
        ..
    }) = expr
    {
        collect_http_methods(receiver, out);
        push_verb_if_valid(&method.to_string(), args, out);
    }
}

fn parse_route_call(node: &ExprMethodCall) -> Option<ExtractedRoute> {
    if node.method != "route" || node.args.len() < 2 {
        return None;
    }
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = &node.args[0]
    else {
        return None;
    };
    let mut methods = Vec::new();
    collect_http_methods(&node.args[1], &mut methods);
    let (method, handler) = methods.into_iter().next()?;
    Some(ExtractedRoute {
        path: lit_str.value(),
        method,
        handler,
    })
}

fn parse_nest_call(node: &ExprMethodCall) -> Option<(String, String)> {
    if node.method != "nest" || node.args.len() < 2 {
        return None;
    }
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = &node.args[0]
    else {
        return None;
    };
    let Expr::Call(ExprCall { func, .. }) = &node.args[1] else {
        return None;
    };
    let Expr::Path(p) = &**func else {
        return None;
    };
    let seg = p.path.segments.last()?;
    Some((lit_str.value(), seg.ident.to_string()))
}

impl<'ast> Visit<'ast> for FnRouteVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        syn::visit::visit_expr_method_call(self, node);
        if let Some(route) = parse_route_call(node) {
            self.routes.push(route);
        } else if let Some(nest) = parse_nest_call(node) {
            self.nests.push(nest);
        }
    }
}

fn append_nested_routes(
    prefix: &str,
    routes: &[ExtractedRoute],
    resolved: &mut Vec<ExtractedRoute>,
) {
    for r in routes {
        let mut full_path = format!("/v1{}{}", prefix, r.path).replace("//", "/");
        if full_path.len() > 1 && full_path.ends_with('/') {
            full_path.pop();
        }
        resolved.push(ExtractedRoute {
            path: full_path,
            method: r.method.clone(),
            handler: r.handler.clone(),
        });
    }
}

pub fn extract_routes_from_router_ast(
    router_src: &str,
) -> Result<Vec<ExtractedRoute>, GraphQueryError> {
    let ast = syn::parse_file(router_src)
        .map_err(|e| GraphQueryError::Validation(format!("Failed to parse router.rs AST: {e}")))?;

    let mut function_routes: HashMap<String, Vec<ExtractedRoute>> = HashMap::new();
    let mut all_nests: Vec<(String, String)> = Vec::new();

    for item in &ast.items {
        let syn::Item::Fn(ItemFn { sig, block, .. }) = item else {
            continue;
        };
        let fn_name = sig.ident.to_string();
        if !fn_name.starts_with("build_") {
            continue;
        }
        let mut visitor = FnRouteVisitor {
            routes: Vec::new(),
            nests: Vec::new(),
        };
        visitor.visit_block(block);
        all_nests.extend(visitor.nests);
        function_routes.insert(fn_name, visitor.routes);
    }

    let mut resolved = Vec::new();
    for (prefix, fn_name) in &all_nests {
        if let Some(routes) = function_routes.get(fn_name) {
            append_nested_routes(prefix, routes, &mut resolved);
        }
    }

    for func in DIRECT_V1_BUILDERS {
        if let Some(routes) = function_routes.get(func) {
            append_nested_routes("", routes, &mut resolved);
        }
    }

    Ok(resolved)
}

fn build_openapi_method_map(openapi_val: &serde_json::Value) -> HashMap<String, HashSet<String>> {
    let mut doc_paths: HashMap<String, HashSet<String>> = HashMap::new();
    let Some(paths_obj) = openapi_val.get("paths").and_then(|v| v.as_object()) else {
        return doc_paths;
    };
    for (raw_path, methods_val) in paths_obj {
        let entry = doc_paths.entry(normalize_route_path(raw_path)).or_default();
        if let Some(m_obj) = methods_val.as_object() {
            entry.extend(m_obj.keys().map(|k| k.to_lowercase()));
        }
    }
    doc_paths
}

pub fn run_route_parity(root_path: &Path, json_output: bool) -> Result<(), GraphQueryError> {
    let router_src = fs::read_to_string(root_path.join("server-rs/src/router.rs"))?;
    let openapi_src = fs::read_to_string(root_path.join("docs/openapi.yaml"))?;

    let routes = extract_routes_from_router_ast(&router_src)?;
    let openapi_val: serde_json::Value = serde_norway::from_str(&openapi_src)
        .map_err(|e| GraphQueryError::Validation(format!("Failed to parse openapi.yaml: {e}")))?;
    let doc_paths = build_openapi_method_map(&openapi_val);

    let mut missing: Vec<String> = Vec::new();
    for r in routes.iter().filter(|r| r.path.starts_with("/v1")) {
        let norm = normalize_route_path(&r.path);
        let method_lower = r.method.to_lowercase();
        let Some(methods) = doc_paths.get(&norm) else {
            missing.push(format!("Route {} missing in openapi.yaml", r.path));
            continue;
        };
        if !methods.contains(&method_lower) {
            let msg = format!(
                "Method {} missing for {} in openapi.yaml",
                method_lower, r.path
            );
            missing.push(msg);
        }
    }

    if json_output {
        let payload = serde_json::json!({
            "routes": routes,
            "missing": missing,
            "ok": missing.is_empty(),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if missing.is_empty() {
        println!(
            "[OK] [ROUTE-PARITY] All {} Axum /v1 routes verified against docs/openapi.yaml via syn AST.",
            routes.len()
        );
        return Ok(());
    }

    for err in &missing {
        eprintln!("[FAIL] [ROUTE-PARITY] {err}");
    }
    Err(GraphQueryError::Validation(format!(
        "{} route parity error(s) detected",
        missing.len()
    )))
}
