use regex::Regex;
use serde_json;
use std::{collections::HashMap, env, fs, process::Command};
use syn::{Attribute, Item, ItemFn};
use vergen::EmitBuilder;

fn main() {
    // Generate build & cargo info; guard git metadata based on worktree presence
    let mut emit_builder = EmitBuilder::builder();
    emit_builder.all_build().all_cargo();

    let in_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if in_git {
        let has_head = Command::new("git")
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if has_head {
            let _ = emit_builder.all_git();
        }
    }

    emit_builder
        .emit()
        .expect("Unable to generate build information");

    // Compute derived build version based on base semver and recent changes
    let base_ver = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let (maj0, min0, pat0) = parse_semver(&base_ver);
    let maj = maj0;
    let mut min = min0;
    let mut pat = pat0; // mut needed for bumping values

    // Heuristic: if git available and HEAD has parent, detect significant changes
    let mut significant = false;
    if in_git {
        if let Ok(out) = Command::new("git")
            .args(["rev-parse", "--verify", "HEAD~1"])
            .output()
        {
            if out.status.success() {
                if let Ok(diff) = Command::new("git")
                    .args(["diff", "--name-only", "HEAD~1..HEAD"])
                    .output()
                {
                    if diff.status.success() {
                        let txt = String::from_utf8_lossy(&diff.stdout);
                        for line in txt.lines() {
                            let p = line.trim();
                            if p.starts_with("src/handlers/")
                                || p == "src/models.rs"
                                || p == "src/db_manager.rs"
                                || p.starts_with("../src/entities/")
                            {
                                significant = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Bump patch every build; bump minor if significant
    pat = pat.saturating_add(1);
    if significant {
        min = min.saturating_add(1);
    }
    let derived_version = format!("{}.{}.{}", maj, min, pat);
    println!("cargo:rustc-env=APP_BUILD_VERSION={}", derived_version);

    // Pass along description from Cargo (Cargo sets CARGO_PKG_DESCRIPTION if present)
    if let Ok(desc) = env::var("CARGO_PKG_DESCRIPTION") {
        println!("cargo:rustc-env=APP_PKG_DESCRIPTION={}", desc);
    }

    // Compute platform-specific suggested binary basename
    let _target = env::var("TARGET").unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let arch_label = match arch.as_str() {
        "x86_64" => "64",
        "aarch64" => "arm64",
        "arm" => "arm",
        _ => arch.as_str(),
    };

    let mut os_label = os.clone();
    let mut os_ver = String::new();

    if os == "linux" {
        // Try /etc/os-release
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            let mut id = String::new();
            let mut ver = String::new();
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("ID=") {
                    id = rest.trim_matches('"').to_string();
                } else if let Some(rest) = line.strip_prefix("VERSION_ID=") {
                    ver = rest.trim_matches('"').to_string();
                }
            }
            if !id.is_empty() {
                os_label = id;
            }
            os_ver = ver;
        }
    } else if os == "macos" || os == "darwin" {
        os_label = "darwin".into();
        // Try sw_vers
        if let Ok(out) = Command::new("sw_vers").arg("-productVersion").output() {
            if out.status.success() {
                os_ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    } else if os == "windows" {
        os_label = "windows".into();
        // Windows version is not reliably available here; default to 11 if unknown
        os_ver = "11".into();
    }

    let _sep_ver = if os_ver.is_empty() {
        String::new()
    } else {
        format!("{}_", os_ver.replace('.', "_"))
    };
    let mut base = format!(
        "{}{}__{}",
        os_label,
        if os_ver.is_empty() {
            String::new()
        } else {
            format!("_{}", os_ver.replace('.', "_"))
        },
        arch_label
    );
    if os_label == "windows" {
        base.push_str(".exe");
    }

    println!("cargo:rustc-env=APP_BIN_FILENAME={}", base);

    // Route metadata generation (auto-doc)
    println!("cargo:rerun-if-changed=src/handlers");
    println!("cargo:rerun-if-changed=src/main.rs");
    if let Err(e) = generate_route_metadata() {
        eprintln!("cargo:warning=route metadata generation skipped: {}", e);
    }
}

fn parse_semver(s: &str) -> (u64, u64, u64) {
    let mut maj = 0;
    let mut min = 0;
    let mut pat = 0;
    let parts: Vec<&str> = s.split('.').collect();
    if !parts.is_empty() {
        maj = parts[0].parse().unwrap_or(0);
    }
    if parts.len() > 1 {
        min = parts[1].parse().unwrap_or(0);
    }
    if parts.len() > 2 {
        pat = parts[2].parse().unwrap_or(0);
    }
    (maj, min, pat)
}

// -----------------------------------------------------------------------------
// Route metadata generation (build-time)
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct RouteMetadata {
    method: String,
    path: String,
    handler: String,
    file: String,
    line: usize,
    description: String,
    feature_gate: Option<String>,
    auth_required: bool,
    tenant_required: bool,
    sample_request: Option<String>,
    sample_response: Option<String>,
}

fn generate_route_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let main_content = fs::read_to_string("src/main.rs")?;
    let scope_map = extract_scopes_from_main(&main_content);

    let regex_routes = scan_routes(&scope_map)?;
    let ast_routes = scan_routes_ast(&scope_map)?;
    let routes = merge_routes(regex_routes, ast_routes);
    let routes = enrich_samples(routes);

    // Extract type metadata from models
    let type_metadata = extract_type_metadata()?;

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = std::path::Path::new(&out_dir).join("route_metadata.rs");
    fs::write(dest_path, render_route_metadata(&routes))?;

    // Write combined metadata JSON for codegen
    let codegen_path = std::path::Path::new(&out_dir).join("codegen_metadata.json");
    let combined_metadata = create_combined_metadata(&routes, &type_metadata)?;
    fs::write(
        codegen_path,
        serde_json::to_string_pretty(&combined_metadata)?,
    )?;

    println!(
        "cargo:warning=Generated codegen metadata with {} routes, {} types, {} enums",
        routes.len(),
        type_metadata.types.len(),
        type_metadata.enums.len()
    );

    Ok(())
}

fn scan_routes(
    scope_map: &HashMap<String, Vec<String>>,
) -> Result<Vec<RouteMetadata>, Box<dyn std::error::Error>> {
    let route_re = Regex::new(
        r#"(?m)^[ \t]*#\[(get|post|put|delete|patch)\("([^"]+)"\)\][ \t]*\n[ \t]*pub async fn ([a-zA-Z_][a-zA-Z0-9_]*)"#,
    )?;
    let mut out = Vec::new();

    for entry in glob::glob("src/handlers/*.rs")? {
        let file_path = entry?;
        let content = fs::read_to_string(&file_path)?;
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let captures: Vec<_> = route_re.captures_iter(&content).collect();
        for (idx, cap) in captures.iter().enumerate() {
            let method = cap.get(1).unwrap().as_str().to_uppercase();
            let raw_path = cap.get(2).unwrap().as_str().to_string();
            let handler_name = cap.get(3).unwrap().as_str().to_string();
            let start = cap.get(0).unwrap().start();
            let end = captures
                .get(idx + 1)
                .map(|n| n.get(0).unwrap().start())
                .unwrap_or_else(|| content.len());

            let doc = extract_doc_comment(&content, start);
            let samples = parse_samples(&doc);
            let feature_gate = parse_feature_tag(&doc);
            let body = &content[start..end];
            let auth_required = infer_auth_required(body, &doc);
            let tenant_required = infer_tenant_required(body, &doc);
            let line = content[..start].lines().count() + 1;

            let handler_key = format!("handlers::{}::{}", file_stem, handler_name);
            let scopes = scope_map.get(&handler_key).cloned().unwrap_or_default();
            let full_path = apply_scopes(&scopes, &raw_path);

            out.push(RouteMetadata {
                method,
                path: full_path,
                handler: handler_key,
                file: file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string(),
                line,
                description: doc,
                feature_gate,
                auth_required,
                tenant_required,
                sample_request: samples.0,
                sample_response: samples.1,
            });
        }
    }

    Ok(out)
}

fn scan_routes_ast(
    scope_map: &HashMap<String, Vec<String>>,
) -> Result<Vec<RouteMetadata>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in glob::glob("src/handlers/*.rs")? {
        let file_path = entry?;
        let content = fs::read_to_string(&file_path)?;
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let ast = match syn::parse_file(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };

        for item in ast.items {
            if let Item::Fn(func) = item {
                if let Some((method, raw_path)) = actix_attr(&func.attrs) {
                    let doc = extract_doc_from_attrs(&func.attrs);
                    let samples = parse_samples(&doc);
                    let feature_gate = parse_feature_tag(&doc);
                    let body_src = snippet_from_span(&content, &func);
                    let auth_required = infer_auth_required(&body_src, &doc);
                    let tenant_required = infer_tenant_required(&body_src, &doc);
                    let handler_key =
                        format!("handlers::{}::{}", file_stem, func.sig.ident.to_string());
                    let scopes = scope_map.get(&handler_key).cloned().unwrap_or_default();
                    let path = apply_scopes(&scopes, &raw_path);
                    out.push(RouteMetadata {
                        method: method.to_uppercase(),
                        path,
                        handler: handler_key,
                        file: file_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        line: 0,
                        description: doc,
                        feature_gate,
                        auth_required,
                        tenant_required,
                        sample_request: samples.0,
                        sample_response: samples.1,
                    });
                }
            }
        }
    }
    Ok(out)
}

fn extract_doc_comment(content: &str, start: usize) -> String {
    let mut doc_lines = Vec::new();
    for line in content[..start].lines().rev() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("///") {
            doc_lines.push(trimmed.trim_start_matches("///").trim());
        } else if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        } else {
            break;
        }
    }
    doc_lines.reverse();
    doc_lines.join(" ")
}

fn parse_feature_tag(doc: &str) -> Option<String> {
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#feature[") {
            if let Some(end) = rest.strip_suffix(']') {
                return Some(end.trim().to_string());
            }
        }
        if let Some(idx) = trimmed.find("@feature ") {
            return Some(trimmed[idx + 9..].trim().to_string());
        }
    }
    None
}

fn extract_doc_from_attrs(attrs: &[Attribute]) -> String {
    let mut docs = Vec::new();
    for attr in attrs {
        let meta = attr.meta.clone();
        if let syn::Meta::NameValue(nv) = meta {
            if nv.path.is_ident("doc") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = nv.value
                {
                    docs.push(s.value());
                }
            }
        }
    }
    docs.join(" ")
}

fn actix_attr(attrs: &[Attribute]) -> Option<(String, String)> {
    for attr in attrs {
        let meta = attr.meta.clone();
        if let syn::Meta::List(list) = meta {
            let ident = list.path.get_ident().map(|i| i.to_string());
            let ident = match ident {
                Some(v) => v,
                None => continue,
            };
            if !["get", "post", "put", "delete", "patch"].contains(&ident.as_str()) {
                continue;
            }
            if let Ok(lit) = list.parse_args::<syn::LitStr>() {
                return Some((ident, lit.value()));
            }
        }
    }
    None
}

fn snippet_from_span(_content: &str, _func: &ItemFn) -> String {
    // Span-to-snippet mapping is optional here; we reuse doc-based inference for AST scan.
    String::new()
}

fn infer_auth_required(body: &str, doc: &str) -> bool {
    if doc.contains("@auth required") {
        return true;
    }
    let needles = [
        "get_user_id",
        "get_claims",
        "guard_api",
        "require_claims",
        "ensure_admin",
        "validate_token",
        "extract_token",
    ];
    needles.iter().any(|n| body.contains(n))
}

fn infer_tenant_required(body: &str, doc: &str) -> bool {
    if doc.contains("@tenant required") {
        return true;
    }
    let needles = ["X-Company-Id", "require_company", "company_from_header"];
    needles.iter().any(|n| body.contains(n))
}

fn parse_samples(doc: &str) -> (Option<String>, Option<String>) {
    let mut sample_req = None;
    let mut sample_res = None;
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("@sample-request") {
            let payload = rest.trim().trim_start_matches(':').trim();
            if !payload.is_empty() {
                sample_req = Some(payload.to_string());
            }
        }
        if let Some(rest) = trimmed.strip_prefix("@sample-response") {
            let payload = rest.trim().trim_start_matches(':').trim();
            if !payload.is_empty() {
                sample_res = Some(payload.to_string());
            }
        }
    }
    (sample_req, sample_res)
}

fn enrich_samples(mut routes: Vec<RouteMetadata>) -> Vec<RouteMetadata> {
    for r in routes.iter_mut() {
        if r.sample_request.is_none() {
            r.sample_request = infer_sample_request(r);
        }
        if r.sample_response.is_none() {
            r.sample_response = infer_sample_response(r);
        }
    }
    routes
}

fn infer_sample_request(route: &RouteMetadata) -> Option<String> {
    let params: Vec<String> = Regex::new(r"\{([^}/]+)\}")
        .unwrap()
        .captures_iter(&route.path)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();

    let mut obj = serde_json::Map::new();
    if !params.is_empty() {
        let mut path_params = serde_json::Map::new();
        for p in params {
            path_params.insert(p, serde_json::Value::String("example".into()));
        }
        obj.insert("pathParams".into(), serde_json::Value::Object(path_params));
    }

    let mut needs_body = matches!(
        route.method.to_uppercase().as_str(),
        "POST" | "PUT" | "PATCH"
    );
    if route.path.to_lowercase().contains("login")
        || route.path.to_lowercase().contains("register")
        || route.path.to_lowercase().contains("auth")
    {
        needs_body = true;
    }

    if needs_body {
        obj.insert("body".into(), serde_json::json!({}));
    }

    if obj.is_empty() {
        None
    } else {
        serde_json::to_string(&serde_json::Value::Object(obj)).ok()
    }
}

fn infer_sample_response(_route: &RouteMetadata) -> Option<String> {
    Some("{\"status\":\"ok\"}".to_string())
}

fn merge_routes(
    mut regex_routes: Vec<RouteMetadata>,
    ast_routes: Vec<RouteMetadata>,
) -> Vec<RouteMetadata> {
    let mut map: HashMap<String, RouteMetadata> = HashMap::new();
    for r in regex_routes.drain(..) {
        let key = format!("{}::{} {}", r.handler, r.method, r.path);
        map.insert(key, r);
    }
    for a in ast_routes {
        let key = format!("{}::{} {}", a.handler, a.method, a.path);
        if let Some(existing) = map.get_mut(&key) {
            if existing.description.is_empty() && !a.description.is_empty() {
                existing.description = a.description.clone();
            }
            if existing.feature_gate.is_none() {
                existing.feature_gate = a.feature_gate.clone();
            }
            existing.auth_required |= a.auth_required;
            existing.tenant_required |= a.tenant_required;
            if existing.line == 0 && a.line != 0 {
                existing.line = a.line;
            }
        } else {
            map.insert(key, a);
        }
    }
    let mut merged: Vec<_> = map.into_values().collect();
    merged.sort_by(|a, b| a.path.cmp(&b.path).then(a.method.cmp(&b.method)));
    merged
}

fn apply_scopes(scopes: &[String], path: &str) -> String {
    let mut full = path.to_string();
    for scope in scopes.iter().rev() {
        if scope.is_empty() {
            continue;
        }
        let scope_norm = if scope.starts_with('/') {
            scope.trim_end_matches('/').to_string()
        } else {
            format!("/{}", scope.trim_end_matches('/'))
        };
        let tail = full.trim_start_matches('/');
        full = format!("{}/{}", scope_norm, tail);
    }
    full
}

fn render_route_metadata(routes: &[RouteMetadata]) -> String {
    let mut out = String::new();
    out.push_str("// @generated by build.rs - DO NOT EDIT\n");
    out.push_str("#[derive(Debug, Clone, serde::Serialize)]\n");
    out.push_str("pub struct RouteInfo {\n");
    out.push_str("    pub method: &'static str,\n");
    out.push_str("    pub path: &'static str,\n");
    out.push_str("    pub handler: &'static str,\n");
    out.push_str("    pub file: &'static str,\n");
    out.push_str("    pub line: usize,\n");
    out.push_str("    pub description: &'static str,\n");
    out.push_str("    pub feature_gate: Option<&'static str>,\n");
    out.push_str("    pub auth_required: bool,\n");
    out.push_str("    pub tenant_required: bool,\n");
    out.push_str("    pub sample_request: Option<&'static str>,\n");
    out.push_str("    pub sample_response: Option<&'static str>,\n");
    out.push_str("}\n\n");

    out.push_str("pub const ROUTE_METADATA: &[RouteInfo] = &[\n");
    for route in routes {
        out.push_str("    RouteInfo {\n");
        out.push_str(&format!("        method: \"{}\",\n", escape(&route.method)));
        out.push_str(&format!("        path: \"{}\",\n", escape(&route.path)));
        out.push_str(&format!(
            "        handler: \"{}\",\n",
            escape(&route.handler)
        ));
        out.push_str(&format!("        file: \"{}\",\n", escape(&route.file)));
        out.push_str(&format!("        line: {},\n", route.line));
        out.push_str(&format!(
            "        description: \"{}\",\n",
            escape(&route.description)
        ));
        match &route.feature_gate {
            Some(f) => out.push_str(&format!("        feature_gate: Some(\"{}\"),\n", escape(f))),
            None => out.push_str("        feature_gate: None,\n"),
        }
        out.push_str(&format!(
            "        auth_required: {},\n",
            route.auth_required
        ));
        out.push_str(&format!(
            "        tenant_required: {},\n",
            route.tenant_required
        ));
        match &route.sample_request {
            Some(s) => out.push_str(&format!(
                "        sample_request: Some(\"{}\"),\n",
                escape(s)
            )),
            None => out.push_str("        sample_request: None,\n"),
        }
        match &route.sample_response {
            Some(s) => out.push_str(&format!(
                "        sample_response: Some(\"{}\"),\n",
                escape(s)
            )),
            None => out.push_str("        sample_response: None,\n"),
        }
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");
    out.push_str(&format!(
        "pub const ROUTE_COUNT: usize = {};\n",
        routes.len()
    ));
    out
}

fn escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn paren_depths(content: &str) -> Vec<i32> {
    let mut depths = Vec::with_capacity(content.len() + 1);
    depths.push(0);
    for b in content.bytes() {
        let mut depth = *depths.last().unwrap();
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
        }
        depths.push(depth);
    }
    depths
}

fn extract_scopes_from_main(main_content: &str) -> HashMap<String, Vec<String>> {
    let scope_re = Regex::new(r#"web::scope\("([^"]+)"\)"#).unwrap();
    let service_re =
        Regex::new(r#"\.service\(handlers::([a-zA-Z0-9_]+)::([a-zA-Z0-9_]+)\)"#).unwrap();

    #[derive(Debug)]
    enum Event {
        Scope { path: String },
        Service { handler: String },
    }

    let depths = paren_depths(main_content);
    let mut events: Vec<(usize, Event)> = Vec::new();

    for cap in scope_re.captures_iter(main_content) {
        let pos = cap.get(0).unwrap().start();
        events.push((
            pos,
            Event::Scope {
                path: cap[1].to_string(),
            },
        ));
    }

    for cap in service_re.captures_iter(main_content) {
        let pos = cap.get(0).unwrap().start();
        let handler = format!("handlers::{}::{}", &cap[1], &cap[2]);
        events.push((pos, Event::Service { handler }));
    }

    events.sort_by_key(|e| e.0);

    let mut stack: Vec<(i32, String)> = Vec::new();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for (pos, event) in events {
        let depth = depths
            .get(pos)
            .copied()
            .unwrap_or_else(|| *depths.last().unwrap_or(&0));

        while let Some((d, _)) = stack.last() {
            if *d > depth {
                stack.pop();
            } else {
                break;
            }
        }

        match event {
            Event::Scope { path } => {
                // Keep scope alive until depth falls below this threshold
                stack.push((depth + 1, path));
            }
            Event::Service { handler } => {
                let scopes: Vec<String> = stack.iter().map(|(_, p)| p.clone()).collect();
                map.insert(handler, scopes);
            }
        }
    }

    map
}

// -----------------------------------------------------------------------------
// Type metadata extraction (for TypeScript codegen)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TypeMetadata {
    types: Vec<serde_json::Value>,
    enums: Vec<serde_json::Value>,
}

fn extract_type_metadata() -> Result<TypeMetadata, Box<dyn std::error::Error>> {
    let mut types = Vec::new();
    let mut enums = Vec::new();

    // Scan all .rs files in src/models/
    for entry in glob::glob("src/models/**/*.rs")? {
        let file_path = entry?;
        let content = fs::read_to_string(&file_path)?;

        let ast = match syn::parse_file(&content) {
            Ok(f) => f,
            Err(_) => continue, // Skip files that don't parse
        };

        for item in ast.items {
            match item {
                syn::Item::Struct(s) if has_serde_derives(&s.attrs) => {
                    if let Ok(type_info) = extract_struct_metadata(&s) {
                        types.push(type_info);
                    }
                }
                syn::Item::Enum(e) if has_serde_derives(&e.attrs) => {
                    if let Ok(enum_info) = extract_enum_metadata(&e) {
                        enums.push(enum_info);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(TypeMetadata { types, enums })
}

fn has_serde_derives(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if let syn::Meta::List(ref list) = attr.meta {
            if list.path.is_ident("derive") {
                let tokens = list.tokens.to_string();
                if tokens.contains("Serialize") || tokens.contains("Deserialize") {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_struct_metadata(
    s: &syn::ItemStruct,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let struct_name = s.ident.to_string();
    let mut fields = Vec::new();

    let serde_directives = parse_serde_directives(&s.attrs);

    if let syn::Fields::Named(ref named_fields) = s.fields {
        for field in &named_fields.named {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let rust_type = type_to_string(&field.ty);
            let optional = is_option_type(&field.ty);
            let serde_rename = get_serde_rename(&field.attrs);

            fields.push(serde_json::json!({
                "name": serde_rename.unwrap_or(field_name),
                "rust_type": rust_type,
                "optional": optional
            }));
        }
    }

    Ok(serde_json::json!({
        "name": struct_name,
        "kind": "struct",
        "fields": fields,
        "serde_directives": serde_directives
    }))
}

fn extract_enum_metadata(
    e: &syn::ItemEnum,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let enum_name = e.ident.to_string();
    let serde_directives = parse_serde_directives(&e.attrs);

    let mut variants = Vec::new();
    for variant in &e.variants {
        let variant_name = variant.ident.to_string();

        match &variant.fields {
            syn::Fields::Unit => {
                // Simple unit variant
                variants.push(serde_json::json!({
                    "name": variant_name,
                    "fields": []
                }));
            }
            syn::Fields::Named(fields) => {
                // Tagged union with named fields
                let mut variant_fields = Vec::new();
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let rust_type = type_to_string(&field.ty);
                    variant_fields.push(serde_json::json!({
                        "name": field_name,
                        "type": rust_type
                    }));
                }
                variants.push(serde_json::json!({
                    "name": variant_name,
                    "fields": variant_fields
                }));
            }
            syn::Fields::Unnamed(fields) => {
                // Tuple variant
                let mut variant_fields = Vec::new();
                for (idx, field) in fields.unnamed.iter().enumerate() {
                    let rust_type = type_to_string(&field.ty);
                    variant_fields.push(serde_json::json!({
                        "name": format!("_{}", idx),
                        "type": rust_type
                    }));
                }
                variants.push(serde_json::json!({
                    "name": variant_name,
                    "fields": variant_fields
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "name": enum_name,
        "variants": variants,
        "serde_directives": serde_directives
    }))
}

fn parse_serde_directives(attrs: &[syn::Attribute]) -> serde_json::Value {
    let mut directives = serde_json::Map::new();

    for attr in attrs {
        if let syn::Meta::List(ref list) = attr.meta {
            if list.path.is_ident("serde") {
                let tokens = list.tokens.to_string();

                // Parse common serde attributes
                if tokens.contains("rename_all") {
                    if let Some(val) = extract_quoted_value(&tokens, "rename_all") {
                        directives.insert("rename_all".to_string(), serde_json::Value::String(val));
                    }
                }

                if tokens.contains("tag") {
                    if let Some(val) = extract_quoted_value(&tokens, "tag") {
                        directives.insert("tag".to_string(), serde_json::Value::String(val));
                    }
                }

                if tokens.contains("deny_unknown_fields") {
                    directives.insert(
                        "deny_unknown_fields".to_string(),
                        serde_json::Value::Bool(true),
                    );
                }
            }
        }
    }

    serde_json::Value::Object(directives)
}

fn get_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let syn::Meta::List(ref list) = attr.meta {
            if list.path.is_ident("serde") {
                let tokens = list.tokens.to_string();
                if let Some(val) = extract_quoted_value(&tokens, "rename") {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn extract_quoted_value(tokens: &str, key: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*"([^"]+)""#, key);
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(cap) = re.captures(tokens) {
            return Some(cap[1].to_string());
        }
    }
    None
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segments: Vec<String> = type_path
                .path
                .segments
                .iter()
                .map(|s| {
                    let ident = s.ident.to_string();
                    if let syn::PathArguments::AngleBracketed(ref args) = s.arguments {
                        let inner_types: Vec<String> = args
                            .args
                            .iter()
                            .filter_map(|arg| {
                                if let syn::GenericArgument::Type(ref ty) = arg {
                                    Some(type_to_string(ty))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if inner_types.is_empty() {
                            ident
                        } else {
                            format!("{}<{}>", ident, inner_types.join(", "))
                        }
                    } else {
                        ident
                    }
                })
                .collect();
            segments.join("::")
        }
        syn::Type::Reference(type_ref) => {
            format!("&{}", type_to_string(&type_ref.elem))
        }
        _ => "Unknown".to_string(),
    }
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn create_combined_metadata(
    routes: &[RouteMetadata],
    type_metadata: &TypeMetadata,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let routes_json: Vec<serde_json::Value> = routes
        .iter()
        .map(|r| {
            serde_json::json!({
                "method": r.method,
                "path": r.path,
                "handler": r.handler,
                "file": r.file,
                "line": r.line,
                "description": r.description,
                "feature_gate": r.feature_gate,
                "auth_required": r.auth_required,
                "tenant_required": r.tenant_required,
                "sample_request": r.sample_request,
                "sample_response": r.sample_response,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "routes": routes_json,
        "types": type_metadata.types,
        "enums": type_metadata.enums
    }))
}
