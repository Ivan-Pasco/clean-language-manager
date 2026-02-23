//! Code generation module for Clean Framework
//!
//! Generates main.cln from discovered project structure:
//! - Handler functions for each route
//! - Route registration in start()
//! - Component imports and registry

use crate::core::discovery::{ApiRoute, Component, DiscoveredProject, Layout, PageRoute};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Sanitize a name for use as a Clean identifier (function/variable name)
/// Replaces hyphens with underscores and removes other invalid characters
fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .map(|c| if c == '-' { '_' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Code generation options
#[derive(Debug, Default)]
pub struct CodegenOptions {
    /// Include debug comments in generated code
    pub debug_comments: bool,
    /// Generate component registry JSON
    pub generate_registry: bool,
}

/// Project configuration parsed from config.cln
#[derive(Debug)]
pub struct ProjectConfig {
    /// Server port (default 3000)
    pub port: u16,
    /// Explicit import paths from config (imported, no route extraction)
    pub imports: Vec<String>,
    /// Route file paths from config (imported AND route registrations extracted)
    pub routes: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            imports: Vec::new(),
            routes: Vec::new(),
        }
    }
}

/// Parse config.cln for project settings
pub fn parse_project_config(project_dir: &Path) -> ProjectConfig {
    let config_path = project_dir.join("config.cln");
    let mut config = ProjectConfig::default();

    if let Ok(content) = fs::read_to_string(config_path) {
        let mut in_imports = false;
        let mut in_routes = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Parse port = NNNN
            if trimmed.starts_with("port") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    if let Ok(p) = val.trim().parse::<u16>() {
                        config.port = p;
                    }
                }
                in_imports = false;
                in_routes = false;
                continue;
            }

            // Parse imports: block
            if trimmed == "imports:" {
                in_imports = true;
                in_routes = false;
                continue;
            }

            // Parse routes: block
            if trimmed == "routes:" {
                in_routes = true;
                in_imports = false;
                continue;
            }

            if in_imports {
                if trimmed.is_empty() {
                    continue;
                }
                if !trimmed.starts_with('"') && !trimmed.starts_with('\'') {
                    in_imports = false;
                    continue;
                }
                let path = trimmed.trim_matches('"').trim_matches('\'');
                if !path.is_empty() {
                    config.imports.push(path.to_string());
                }
            }

            if in_routes {
                if trimmed.is_empty() {
                    continue;
                }
                if !trimmed.starts_with('"') && !trimmed.starts_with('\'') {
                    in_routes = false;
                    continue;
                }
                let path = trimmed.trim_matches('"').trim_matches('\'');
                if !path.is_empty() {
                    config.routes.push(path.to_string());
                }
            }
        }
    }

    config
}

/// Result of code generation
#[derive(Debug)]
pub struct GeneratedCode {
    /// Generated main.cln content
    pub main_cln: String,
    /// Component registry JSON (if requested)
    pub component_registry: Option<String>,
    /// List of files to compile (main.cln + dependencies)
    pub compile_order: Vec<String>,
}

/// Generate main.cln and related files from discovered project
pub fn generate_code(
    project: &DiscoveredProject,
    project_dir: &Path,
    options: &CodegenOptions,
) -> Result<GeneratedCode> {
    // Parse project config for port, imports, routes, etc.
    let config = parse_project_config(project_dir);

    // Scan route files for handler indices and route registrations
    let (handler_offset, imported_route_lines) = scan_route_files(&config.routes, project_dir);
    let mut handler_index: usize = handler_offset;

    let mut main_cln = String::new();

    // Generate plugins and import sections
    main_cln.push_str(&generate_imports(project, project_dir, &config)?);

    // Generate model classes
    if !project.models.is_empty() {
        main_cln.push_str("\n// Database models\n");
        for model in &project.models {
            main_cln.push_str(&generate_model_import(&model.source_file, project_dir)?);
        }
    }

    // Generate component classes (if any need server-side rendering)
    if !project.components.is_empty() && options.debug_comments {
        main_cln.push_str("\n// Components available: ");
        let tags: Vec<&str> = project.components.iter().map(|c| c.tag.as_str()).collect();
        main_cln.push_str(&tags.join(", "));
        main_cln.push('\n');
    }

    // Generate start: block BEFORE functions: (compiler requires this order)
    main_cln.push_str(&generate_start_function(
        project,
        options,
        config.port,
        handler_offset,
        &imported_route_lines,
    )?);

    // Generate functions block with handlers
    main_cln.push_str("\nfunctions:\n");

    // _html_escape is provided by the frame.ui plugin (which is always present when
    // components exist), so we do NOT inline it here to avoid duplicate definitions.

    // Generate component render functions FIRST (so page handlers can call them)
    for component in &project.components {
        main_cln.push_str(&generate_component_render_function(component, options)?);
    }

    // Page handlers (with component expansion and layout wrapping)
    for page in &project.pages {
        main_cln.push_str(&generate_page_handler(
            page,
            project_dir,
            handler_index,
            &project.components,
            &project.layouts,
            options,
        )?);
        handler_index += 1;
    }

    // API handlers
    for api in &project.api_routes {
        main_cln.push_str(&generate_api_handler(
            api,
            project_dir,
            handler_index,
            options,
        )?);
        handler_index += 1;
    }

    // Generate component registry if requested
    let component_registry = if options.generate_registry && !project.components.is_empty() {
        Some(generate_component_registry(&project.components)?)
    } else {
        None
    };

    // Build compile order
    let compile_order = build_compile_order(project, project_dir)?;

    Ok(GeneratedCode {
        main_cln,
        component_registry,
        compile_order,
    })
}

/// Extract component props from a `props:` block in the source file
/// Returns a list of (type, name) pairs
fn extract_component_props(content: &str) -> Vec<(String, String)> {
    let mut props = Vec::new();
    let mut in_props = false;
    let mut props_base_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "props:" {
            in_props = true;
            props_base_indent = line.len() - line.trim_start().len();
            continue;
        }

        if in_props {
            let current_indent = line.len() - line.trim_start().len();

            if trimmed.is_empty() {
                continue;
            }

            // End of props block
            if current_indent <= props_base_indent {
                break;
            }

            // Parse "type name" or "type name = default"
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let prop_type = parts[0].to_string();
                let prop_name = parts[1].trim_end_matches('=').trim().to_string();
                props.push((prop_type, prop_name));
            }
        }
    }

    props
}

/// Generate a component render function from its source file
fn generate_component_render_function(
    component: &Component,
    options: &CodegenOptions,
) -> Result<String> {
    let mut output = String::new();

    if options.debug_comments {
        output.push_str(&format!(
            "\t// Component: <{}> from {}\n",
            component.tag,
            component
                .source_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    }

    // Read component source
    let content = fs::read_to_string(&component.source_file).with_context(|| {
        format!(
            "Failed to read component: {}",
            component.source_file.display()
        )
    })?;

    // Extract props
    let props = extract_component_props(&content);

    // Extract render function body
    let mut render_body = extract_component_render_body(&content)?;

    // Replace this.prop with prop name for standalone functions
    for (_prop_type, prop_name) in &props {
        let this_ref = format!("this.{}", prop_name);
        render_body = render_body.replace(&this_ref, prop_name);
    }

    // Generate function signature with props as parameters
    let sanitized_name = sanitize_identifier(&component.class_name);
    if props.is_empty() {
        output.push_str(&format!(
            "\tstring __component_{}_render()\n",
            sanitized_name
        ));
    } else {
        let params: Vec<String> = props.iter().map(|(t, n)| format!("{} {}", t, n)).collect();
        output.push_str(&format!(
            "\tstring __component_{}_render({})\n",
            sanitized_name,
            params.join(", ")
        ));
    }
    output.push_str(&indent_code(&render_body, 2));
    output.push_str("\n\n");

    Ok(output)
}

/// Extract the render function body from a component source file
///
/// Tries two strategies:
/// 1. Look for a `string render()` function and extract its body
/// 2. Look for an `html:` block and convert it to string concatenation
fn extract_component_render_body(content: &str) -> Result<String> {
    // Strategy 1: Look for string render() function
    let mut in_render = false;
    let mut render_body = String::new();
    let mut base_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for render() function start
        if trimmed.starts_with("string render()") || trimmed.contains("string render()") {
            in_render = true;
            // Calculate base indentation
            base_indent = line.len() - line.trim_start().len();
            continue;
        }

        if in_render {
            let current_indent = line.len() - line.trim_start().len();

            if trimmed.is_empty() {
                continue;
            }

            // If we hit a line at the same or lower indentation, stop
            if current_indent <= base_indent {
                break;
            }

            render_body.push_str(trimmed);
            render_body.push('\n');
        }
    }

    if !render_body.is_empty() {
        return Ok(render_body.trim_end().to_string());
    }

    // Strategy 2: Look for html: block and convert to string concatenation
    let mut in_html = false;
    let mut html_lines = Vec::new();
    let mut html_base_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "html:" {
            in_html = true;
            html_base_indent = line.len() - line.trim_start().len();
            continue;
        }

        if in_html {
            let current_indent = line.len() - line.trim_start().len();

            if trimmed.is_empty() {
                continue;
            }

            // If we hit a line at the same or lower indentation, stop
            if current_indent <= html_base_indent {
                break;
            }

            html_lines.push(trimmed.to_string());
        }
    }

    if !html_lines.is_empty() {
        // Convert html: block lines to string concatenation
        let mut output = String::new();
        output.push_str("string html = \"");

        for (i, line) in html_lines.iter().enumerate() {
            if i == 0 {
                output.push_str(&escape_html_line(line));
            } else {
                output.push_str("\"\n");
                output.push_str(&format!("html = html + \"{}", escape_html_line(line)));
            }
        }

        output.push_str("\"\n");
        output.push_str("return html");
        return Ok(output);
    }

    // No render body found - return placeholder
    Ok("return \"\"".to_string())
}

/// Escape a single HTML line for embedding in a Clean string literal
///
/// Handles interpolation syntax:
/// - `{{expr}}` → `" + expr + "` (legacy double-brace)
/// - `{!expr}` → `" + expr + "` (raw interpolation, no escaping)
/// - `{expr}` → `" + _html_escape(expr) + "` (safe interpolation)
fn escape_html_line(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\t' => result.push_str("\\t"),
            '{' if chars.peek() == Some(&'{') => {
                // Legacy {{expr}} interpolation
                chars.next();
                let mut var_name = String::new();
                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next();
                        break;
                    }
                    var_name.push(vc);
                }
                result.push_str("\" + ");
                result.push_str(var_name.trim());
                result.push_str(" + \"");
            }
            '{' => {
                // Single-brace interpolation: {expr} or {!expr}
                let raw = chars.peek() == Some(&'!');
                if raw {
                    chars.next(); // consume '!'
                }
                let mut expr = String::new();
                for vc in chars.by_ref() {
                    if vc == '}' {
                        break;
                    }
                    expr.push(vc);
                }
                let expr = expr.trim();
                if raw {
                    result.push_str("\" + ");
                    result.push_str(expr);
                    result.push_str(" + \"");
                } else {
                    result.push_str("\" + _html_escape(");
                    result.push_str(expr);
                    result.push_str(") + \"");
                }
            }
            '}' => result.push_str("\\}"),
            _ => result.push(c),
        }
    }

    result
}

/// Extract the data block from a page's <script type="text/clean"> section
fn extract_page_data_block(content: &str) -> String {
    let mut data_block = String::new();
    let mut in_script = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<script type=\"text/clean\">")
            || trimmed.contains("<script type='text/clean'>")
        {
            in_script = true;
            continue;
        }
        if in_script {
            if trimmed.contains("</script>") {
                break;
            }
            if !trimmed.starts_with("data:") && !trimmed.is_empty() {
                data_block.push_str(trimmed);
                data_block.push('\n');
            }
        }
    }

    data_block
}

/// Scan route files for _http_route() calls and find max handler index
/// Returns (next_available_handler_index, vec_of_route_registration_lines)
fn scan_route_files(routes: &[String], project_dir: &Path) -> (usize, Vec<String>) {
    let mut max_index: Option<usize> = None;
    let mut route_lines = Vec::new();

    for route_path in routes {
        let full_path = project_dir.join(route_path);
        if let Ok(content) = fs::read_to_string(&full_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                // Extract _http_route() calls (skip comments)
                if trimmed.contains("_http_route(") && !trimmed.starts_with("//") {
                    route_lines.push(trimmed.to_string());
                    if let Some(idx) = extract_route_handler_index(trimmed) {
                        max_index = Some(max_index.map_or(idx, |m: usize| m.max(idx)));
                    }
                }
                // Also check __route_handler_N() function definitions
                if let Some(idx) = extract_handler_def_index(trimmed) {
                    max_index = Some(max_index.map_or(idx, |m: usize| m.max(idx)));
                }
            }
        }
    }

    let next_index = max_index.map_or(0, |m| m + 1);
    (next_index, route_lines)
}

/// Extract handler index from a _http_route() call
/// e.g., `s = _http_route("GET", "/api/health", 0)` → Some(0)
fn extract_route_handler_index(line: &str) -> Option<usize> {
    if let Some(paren_pos) = line.rfind(')') {
        let before_paren = &line[..paren_pos];
        if let Some(comma_pos) = before_paren.rfind(',') {
            let num_str = before_paren[comma_pos + 1..].trim();
            return num_str.parse::<usize>().ok();
        }
    }
    None
}

/// Extract handler index from a __route_handler_N() function definition
/// e.g., `string __route_handler_5()` → Some(5)
fn extract_handler_def_index(line: &str) -> Option<usize> {
    if let Some(start) = line.find("__route_handler_") {
        let after = &line[start + "__route_handler_".len()..];
        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !num_str.is_empty() {
            return num_str.parse::<usize>().ok();
        }
    }
    None
}

/// Check if any source files in the project use database functions
fn project_uses_database(
    project: &DiscoveredProject,
    config: &ProjectConfig,
    project_dir: &Path,
) -> bool {
    let db_patterns = [
        "_db_query",
        "_db_execute",
        "_db_insert",
        "_db_update",
        "_db_delete",
    ];

    // Collect all source files to scan (discovered + config imports/routes)
    let discovered_files: Vec<std::path::PathBuf> = project
        .api_routes
        .iter()
        .map(|r| r.source_file.clone())
        .chain(project.pages.iter().map(|p| p.source_file.clone()))
        .chain(project.lib_modules.iter().map(|l| l.source_file.clone()))
        .collect();

    let config_files: Vec<std::path::PathBuf> = config
        .imports
        .iter()
        .chain(config.routes.iter())
        .map(|p| project_dir.join(p))
        .collect();

    for path in discovered_files.iter().chain(config_files.iter()) {
        if let Ok(content) = fs::read_to_string(path) {
            for pattern in &db_patterns {
                if content.contains(pattern) {
                    return true;
                }
            }
        }
    }

    false
}

/// Generate plugins and import blocks
fn generate_imports(
    project: &DiscoveredProject,
    project_dir: &Path,
    config: &ProjectConfig,
) -> Result<String> {
    let mut output = String::new();
    let mut plugins = Vec::new();

    // Determine which plugins are needed
    let needs_httpserver =
        !project.pages.is_empty() || !project.api_routes.is_empty() || !config.routes.is_empty();
    let needs_data =
        !project.models.is_empty() || project_uses_database(project, config, project_dir);
    let needs_ui = !project.components.is_empty();

    if needs_httpserver {
        plugins.push("frame.httpserver");
    }
    if needs_data {
        plugins.push("frame.data");
    }
    if needs_ui {
        plugins.push("frame.ui");
    }

    if !plugins.is_empty() {
        output.push_str("plugins:\n");
        for plugin in &plugins {
            output.push_str(&format!("\t{}\n", plugin));
        }
    }

    // Generate import: block from config imports, routes, and shared lib modules
    // All paths get ../../ prefix since generated file is at dist/.generated/main.cln
    let mut import_paths: Vec<String> = Vec::new();

    // Add explicit imports from config.cln
    for import in &config.imports {
        let prefixed = format!("../../{}", import);
        if !import_paths.contains(&prefixed) {
            import_paths.push(prefixed);
        }
    }

    // Add route files from config.cln (they also need to be imported)
    for route in &config.routes {
        let prefixed = format!("../../{}", route);
        if !import_paths.contains(&prefixed) {
            import_paths.push(prefixed);
        }
    }

    // Add shared lib modules (auto-discovered from app/shared/lib/)
    for lib in &project.lib_modules {
        let relative = lib
            .source_file
            .strip_prefix(project_dir)
            .unwrap_or(&lib.source_file);
        let prefixed = format!("../../{}", relative.to_string_lossy());
        if !import_paths.contains(&prefixed) {
            import_paths.push(prefixed);
        }
    }

    if !import_paths.is_empty() {
        output.push_str("\nimport:\n");
        for path in &import_paths {
            output.push_str(&format!("\t\"{}\"\n", path));
        }
    }

    if !output.is_empty() {
        output.push('\n');
    }

    Ok(output)
}

/// Extract the layout name from a page's `<page layout="X">` directive
fn extract_page_layout(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<page ") {
            // Extract layout="..." attribute
            if let Some(start) = trimmed.find("layout=\"") {
                let after = &trimmed[start + 8..];
                if let Some(end) = after.find('"') {
                    return Some(after[..end].to_string());
                }
            }
            // Also try single quotes
            if let Some(start) = trimmed.find("layout='") {
                let after = &trimmed[start + 8..];
                if let Some(end) = after.find('\'') {
                    return Some(after[..end].to_string());
                }
            }
        }
    }
    None
}

/// Find a layout by name from discovered layouts
fn find_layout<'a>(layouts: &'a [Layout], name: &str) -> Option<&'a Layout> {
    layouts.iter().find(|l| l.name == name)
}

/// Apply layout wrapping: read layout HTML, replace <slot></slot> with page content
fn apply_layout(
    layout_path: &Path,
    page_html_lines: &[&str],
    components: &[Component],
) -> Result<String> {
    let layout_content = fs::read_to_string(layout_path)
        .with_context(|| format!("Failed to read layout: {}", layout_path.display()))?;

    let layout_lines: Vec<&str> = layout_content.lines().collect();
    let mut merged = Vec::new();

    for layout_line in &layout_lines {
        let trimmed = layout_line.trim();
        if trimmed == "<slot></slot>" || trimmed == "<slot />" || trimmed == "<slot/>" {
            // Replace slot with page content lines
            for page_line in page_html_lines {
                merged.push(*page_line);
            }
        } else {
            merged.push(*layout_line);
        }
    }

    // Convert merged HTML to Clean code (handles component tags and {{var}} interpolation)
    let merged_html = merged.join("\n");
    convert_html_to_clean(&merged_html, components)
}

/// Generate a page handler function
fn generate_page_handler(
    page: &PageRoute,
    project_dir: &Path,
    handler_index: usize,
    components: &[Component],
    layouts: &[Layout],
    options: &CodegenOptions,
) -> Result<String> {
    let mut handler = String::new();

    if options.debug_comments {
        handler.push_str(&format!(
            "\t// Handler for {} (from {})\n",
            page.path,
            page.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&page.source_file)
                .display()
        ));
    }

    handler.push_str(&format!("\tstring __route_handler_{}()\n", handler_index));

    // Extract route parameters
    let params = extract_route_params(&page.path);
    for param in &params {
        handler.push_str(&format!(
            "\t\tstring {} = _req_param(\"{}\")\n",
            param, param
        ));
    }

    // Read page source and check for layout directive
    let page_content = fs::read_to_string(&page.source_file)
        .with_context(|| format!("Failed to read page: {}", page.source_file.display()))?;

    let layout_name = extract_page_layout(&page_content);

    let source = if let Some(ref name) = layout_name {
        if let Some(layout) = find_layout(layouts, name) {
            // Extract data block BEFORE layout merge (Bug 18 fix)
            let data_block = extract_page_data_block(&page_content);
            // Extract page's HTML lines (without script block, page directive, etc.)
            let page_html_lines = extract_page_html_lines(&page_content);
            let layout_code = apply_layout(&layout.source_file, &page_html_lines, components)?;
            // Prepend data block before HTML assembly
            let mut code = String::new();
            if !data_block.is_empty() {
                for line in data_block.lines() {
                    if !line.trim().is_empty() {
                        code.push_str(line);
                        code.push('\n');
                    }
                }
            }
            code.push_str(&layout_code);
            code
        } else {
            // Layout not found — fall back to no layout
            convert_html_to_clean(&page_content, components)?
        }
    } else {
        convert_html_to_clean(&page_content, components)?
    };

    handler.push_str(&indent_code(&source, 2));
    handler.push('\n');

    Ok(handler)
}

/// Extract the HTML template lines from a page (excluding script block and <page> directive)
fn extract_page_html_lines(content: &str) -> Vec<&str> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_script = false;

    for line in &lines {
        let trimmed = line.trim();

        // Skip script blocks
        if trimmed.contains("<script type=\"text/clean\">")
            || trimmed.contains("<script type='text/clean'>")
        {
            in_script = true;
            continue;
        }
        if in_script {
            if trimmed.contains("</script>") {
                in_script = false;
            }
            continue;
        }

        // Skip <page> directive
        if trimmed.starts_with("<page ") && trimmed.ends_with('>') {
            continue;
        }

        // Skip empty leading lines
        if result.is_empty() && trimmed.is_empty() {
            continue;
        }

        result.push(*line);
    }

    result
}

/// Generate an API handler function
fn generate_api_handler(
    api: &ApiRoute,
    project_dir: &Path,
    handler_index: usize,
    options: &CodegenOptions,
) -> Result<String> {
    let mut handler = String::new();

    if options.debug_comments {
        handler.push_str(&format!(
            "\t// API handler for {} {} (from {})\n",
            api.method,
            api.path,
            api.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&api.source_file)
                .display()
        ));
    }

    handler.push_str(&format!("\tstring __route_handler_{}()\n", handler_index));

    // Extract route parameters
    let params = extract_route_params(&api.path);
    for param in &params {
        handler.push_str(&format!(
            "\t\tstring {} = _req_param(\"{}\")\n",
            param, param
        ));
    }

    // Read and inline the API source
    let source = read_api_source(&api.source_file)?;
    handler.push_str(&indent_code(&source, 2));
    handler.push('\n');

    Ok(handler)
}

/// Generate the start: block with route registration
fn generate_start_function(
    project: &DiscoveredProject,
    options: &CodegenOptions,
    port: u16,
    handler_offset: usize,
    imported_route_lines: &[String],
) -> Result<String> {
    let mut start = String::new();

    start.push_str("\nstart:\n");
    start.push_str("\tinteger s = 0\n");

    // Include imported route registrations first (from config routes: files)
    if !imported_route_lines.is_empty() {
        if options.debug_comments {
            start.push_str("\n\t// Imported route registrations\n");
        }
        for line in imported_route_lines {
            // Ensure proper formatting with tab indent and s = prefix
            let trimmed = line.trim();
            if trimmed.starts_with("s =") || trimmed.starts_with("s=") {
                start.push_str(&format!("\t{}\n", trimmed));
            } else if trimmed.contains("_http_route(") {
                start.push_str(&format!("\ts = {}\n", trimmed));
            }
        }
    }

    let mut handler_index = handler_offset;

    if options.debug_comments && !project.pages.is_empty() {
        start.push_str("\n\t// Page routes\n");
    }

    // Register page routes
    for page in &project.pages {
        start.push_str(&format!(
            "\ts = _http_route(\"{}\", \"{}\", {})\n",
            page.method, page.path, handler_index
        ));
        handler_index += 1;
    }

    if options.debug_comments && !project.api_routes.is_empty() {
        start.push_str("\n\t// API routes\n");
    }

    // Register API routes
    for api in &project.api_routes {
        start.push_str(&format!(
            "\ts = _http_route(\"{}\", \"{}\", {})\n",
            api.method, api.path, handler_index
        ));
        handler_index += 1;
    }

    // Start HTTP listener on configured port
    let has_routes = !project.pages.is_empty()
        || !project.api_routes.is_empty()
        || !imported_route_lines.is_empty();
    if has_routes {
        start.push_str(&format!("\ts = _http_listen({})\n", port));
    }

    start.push('\n');

    Ok(start)
}

/// Generate model import/include
fn generate_model_import(source_file: &Path, project_dir: &Path) -> Result<String> {
    let relative = source_file.strip_prefix(project_dir).unwrap_or(source_file);
    Ok(format!("// include: {}\n", relative.display()))
}

/// Generate component registry JSON
fn generate_component_registry(components: &[Component]) -> Result<String> {
    let mut registry = String::from("{\n  \"components\": [\n");

    for (i, component) in components.iter().enumerate() {
        registry.push_str(&format!(
            "    {{\n      \"tag\": \"{}\",\n      \"class\": \"{}\",\n      \"hydration\": \"{}\"\n    }}",
            component.tag, component.class_name, component.hydration
        ));
        if i < components.len() - 1 {
            registry.push(',');
        }
        registry.push('\n');
    }

    registry.push_str("  ]\n}");
    Ok(registry)
}

/// Build list of files to compile in order
fn build_compile_order(project: &DiscoveredProject, project_dir: &Path) -> Result<Vec<String>> {
    let mut order = Vec::new();

    // Models first (they define data structures)
    for model in &project.models {
        order.push(
            model
                .source_file
                .strip_prefix(project_dir)
                .unwrap_or(&model.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Lib modules
    for lib in &project.lib_modules {
        order.push(
            lib.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&lib.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Components
    for component in &project.components {
        order.push(
            component
                .source_file
                .strip_prefix(project_dir)
                .unwrap_or(&component.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Generated main.cln is last
    order.push("dist/.generated/main.cln".to_string());

    Ok(order)
}

/// Extract route parameters from path (e.g., /blog/:slug -> ["slug"])
fn extract_route_params(path: &str) -> Vec<String> {
    let mut params = Vec::new();

    for segment in path.split('/') {
        if let Some(stripped) = segment.strip_prefix(':') {
            params.push(stripped.to_string());
        }
    }

    params
}

/// Read page source file and convert HTML to Clean Language
#[allow(dead_code)]
fn read_page_source(source_file: &Path, components: &[Component]) -> Result<String> {
    let content = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read page file: {}", source_file.display()))?;

    // Convert HTML to Clean Language string concatenation (with component expansion)
    convert_html_to_clean(&content, components)
}

/// Convert HTML content to Clean Language string concatenation code
fn convert_html_to_clean(html: &str, components: &[Component]) -> Result<String> {
    let mut output = String::new();
    let mut lines: Vec<&str> = html.lines().collect();

    // Check for <script type="text/clean"> block - extract data loading
    let mut data_block = String::new();
    let mut in_script = false;
    let mut script_start = 0;
    let mut script_end = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("<script type=\"text/clean\">")
            || trimmed.contains("<script type='text/clean'>")
        {
            in_script = true;
            script_start = i;
        } else if in_script && trimmed.contains("</script>") {
            script_end = i;
            in_script = false;
        } else if in_script {
            // Collect data block lines (skip the data: keyword itself if present)
            if !trimmed.starts_with("data:") && !trimmed.is_empty() {
                data_block.push_str(trimmed);
                data_block.push('\n');
            }
        }
    }

    // Remove script block from lines if found
    if script_end > script_start {
        lines = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < script_start || *i > script_end)
            .map(|(_, l)| *l)
            .collect();
    }

    // Add data loading code first (executable, before HTML string building)
    if !data_block.is_empty() {
        for line in data_block.lines() {
            if !line.trim().is_empty() {
                output.push_str(line);
                output.push('\n');
            }
        }
    }

    // Build HTML as string concatenation
    output.push_str("string html = \"");

    let mut first_line = true;
    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines at start
        if first_line && trimmed.is_empty() {
            continue;
        }
        first_line = false;

        // Skip HTML comments
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            continue;
        }

        // Skip <page> directive tags
        if trimmed.starts_with("<page ") && trimmed.ends_with(">") {
            continue;
        }

        // Close current string and start new concatenation for each line
        if !output.ends_with("\"") {
            output.push_str("\"\n");
            output.push_str("html = html + \"");
        }

        // Check for component tags and expand them
        let expanded = expand_component_tags(trimmed, components);
        output.push_str(&expanded);
    }

    output.push_str("\"\n");
    output.push_str("return html");

    Ok(output)
}

/// Expand component tags in HTML line to function calls
fn expand_component_tags(line: &str, components: &[Component]) -> String {
    let mut result = line.to_string();

    for component in components {
        // Match self-closing tags: <app-header></app-header> or <app-header />
        let self_closing = format!("<{}></{}>", component.tag, component.tag);
        let self_closing_short = format!("<{} />", component.tag);
        let self_closing_nospace = format!("<{}/>", component.tag);

        // Also match just opening/closing if on same line
        let sanitized_name = sanitize_identifier(&component.class_name);
        if result.contains(&self_closing) {
            // Replace with function call
            let replacement = format!("\" + __component_{}_render() + \"", sanitized_name);
            result = result.replace(&self_closing, &replacement);
        } else if result.contains(&self_closing_short) {
            let replacement = format!("\" + __component_{}_render() + \"", sanitized_name);
            result = result.replace(&self_closing_short, &replacement);
        } else if result.contains(&self_closing_nospace) {
            let replacement = format!("\" + __component_{}_render() + \"", sanitized_name);
            result = result.replace(&self_closing_nospace, &replacement);
        }
    }

    // Now escape remaining HTML (but preserve our function call insertions)
    escape_html_for_clean_with_calls(&result)
}

/// Escape HTML content for Clean strings, but preserve function call insertions
fn escape_html_for_clean_with_calls(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.chars().peekable();
    let mut in_function_call = false;

    while let Some(c) = chars.next() {
        // Check for function call marker: " + __component_
        if c == '"' && !in_function_call {
            // Look ahead for function call pattern
            let remaining: String = chars.clone().take(20).collect();
            if remaining.starts_with(" + __component_") {
                // This is a function call insertion - pass through as-is
                result.push(c);
                in_function_call = true;
                continue;
            }
        }

        // Check for end of function call: handle both () and (args) with paren depth
        if in_function_call && c == '(' {
            result.push(c);
            // Consume everything until matching closing paren
            let mut depth = 1;
            while depth > 0 {
                if let Some(nc) = chars.next() {
                    result.push(nc);
                    match nc {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            // After closing paren, look for ` + "`
            if chars.peek() == Some(&' ') {
                result.push(chars.next().unwrap()); // space
                if chars.peek() == Some(&'+') {
                    result.push(chars.next().unwrap()); // +
                    if chars.peek() == Some(&' ') {
                        result.push(chars.next().unwrap()); // space
                        if chars.peek() == Some(&'"') {
                            result.push(chars.next().unwrap()); // "
                            in_function_call = false;
                        }
                    }
                }
            }
            continue;
        }

        if in_function_call {
            // Inside function call - pass through as-is
            result.push(c);
            continue;
        }

        // Normal HTML escaping
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\t' => result.push_str("\\t"),
            '{' if chars.peek() == Some(&'{') => {
                // Handle {{variable}} interpolation
                chars.next(); // consume second {
                let mut var_name = String::new();
                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        break;
                    }
                    var_name.push(vc);
                }
                // Convert {{var}} to Clean string concatenation
                result.push_str("\" + ");
                result.push_str(&var_name);
                result.push_str(" + \"");
            }
            // Escape single braces with backslash for Clean Language
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            _ => result.push(c),
        }
    }

    result
}

/// Escape HTML content for embedding in Clean Language strings
#[allow(dead_code)]
fn escape_html_for_clean(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\t' => result.push_str("\\t"),
            '{' if chars.peek() == Some(&'{') => {
                // Handle {{variable}} interpolation
                chars.next(); // consume second {
                let mut var_name = String::new();
                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        break;
                    }
                    var_name.push(vc);
                }
                // Convert {{var}} to Clean string concatenation
                result.push_str("\" + ");
                result.push_str(&var_name);
                result.push_str(" + \"");
            }
            _ => result.push(c),
        }
    }

    result
}

/// Read API source file and extract the handler body
fn read_api_source(source_file: &Path) -> Result<String> {
    let content = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read API file: {}", source_file.display()))?;

    // Return content as-is
    Ok(content)
}

/// Indent code by the specified number of tabs
fn indent_code(code: &str, tabs: usize) -> String {
    let indent = "\t".repeat(tabs);
    code.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Write generated code to disk
pub fn write_generated_code(generated: &GeneratedCode, output_dir: &Path) -> Result<()> {
    let gen_dir = output_dir.join(".generated");
    fs::create_dir_all(&gen_dir).context("Failed to create .generated directory")?;

    // Write main.cln
    fs::write(gen_dir.join("main.cln"), &generated.main_cln)
        .context("Failed to write generated main.cln")?;

    // Write component registry if present
    if let Some(registry) = &generated.component_registry {
        fs::write(gen_dir.join("components.json"), registry)
            .context("Failed to write component registry")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_route_params() {
        assert_eq!(extract_route_params("/"), Vec::<String>::new());
        assert_eq!(extract_route_params("/about"), Vec::<String>::new());
        assert_eq!(extract_route_params("/blog/:slug"), vec!["slug"]);
        assert_eq!(
            extract_route_params("/users/:id/posts/:postId"),
            vec!["id", "postId"]
        );
    }

    #[test]
    fn test_indent_code() {
        let code = "line1\nline2\nline3";
        let indented = indent_code(code, 2);
        assert_eq!(indented, "\t\tline1\n\t\tline2\n\t\tline3");
    }

    #[test]
    fn test_component_render_no_double_quotes() {
        // Bug 7: html: block conversion should not produce trailing ""
        let component_src = r#"component Hero
    html:
        <section class="hero">
        <div class="container">
        <h1>Hello</h1>
        </div>
        </section>
"#;
        let body = extract_component_render_body(component_src).unwrap();
        // No line should end with "" (double closing quotes)
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("html = html + ") {
                assert!(
                    !trimmed.ends_with("\"\""),
                    "Line has trailing double quotes: {}",
                    trimmed
                );
            }
        }
        // Each concatenation line should end with exactly one "
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("html = html + \"") {
                assert!(
                    trimmed.ends_with('"'),
                    "Line should end with single quote: {}",
                    trimmed
                );
                // Count trailing quotes
                let trailing_quotes = trimmed.chars().rev().take_while(|c| *c == '"').count();
                assert_eq!(
                    trailing_quotes, 1,
                    "Expected 1 trailing quote, got {} in: {}",
                    trailing_quotes, trimmed
                );
            }
        }
    }

    #[test]
    fn test_interpolation_safe_escape() {
        // Bug 9: {expr} should expand to _html_escape(expr)
        let result = escape_html_line("<h3>{this.title}</h3>");
        assert_eq!(result, "<h3>\" + _html_escape(this.title) + \"</h3>");
    }

    #[test]
    fn test_interpolation_raw() {
        // Bug 9: {!expr} should expand to raw expr (no escaping)
        let result = escape_html_line("<div>{!getIcon(this.icon)}</div>");
        assert_eq!(result, "<div>\" + getIcon(this.icon) + \"</div>");
    }

    #[test]
    fn test_interpolation_legacy_double_brace() {
        // {{expr}} should still work (legacy syntax)
        let result = escape_html_line("<span>{{name}}</span>");
        assert_eq!(result, "<span>\" + name + \"</span>");
    }

    #[test]
    fn test_extract_component_props() {
        let src = r#"component ModuleCard
    props:
        string id
        string title
        string difficulty
    html:
        <article>{this.title}</article>
"#;
        let props = extract_component_props(src);
        assert_eq!(props.len(), 3);
        assert_eq!(props[0], ("string".to_string(), "id".to_string()));
        assert_eq!(props[1], ("string".to_string(), "title".to_string()));
        assert_eq!(props[2], ("string".to_string(), "difficulty".to_string()));
    }

    #[test]
    fn test_extract_component_props_empty() {
        let src = r#"component Hero
    html:
        <section>Hello</section>
"#;
        let props = extract_component_props(src);
        assert!(props.is_empty());
    }

    #[test]
    fn test_this_prop_replaced_in_render_body() {
        // Bug 10: this.prop should be replaced with prop name
        let src = r#"component Card
    props:
        string title
        string desc
    html:
        <h3>{this.title}</h3>
        <p>{this.desc}</p>
"#;
        let mut body = extract_component_render_body(src).unwrap();
        let props = extract_component_props(src);
        for (_t, name) in &props {
            body = body.replace(&format!("this.{}", name), name);
        }
        assert!(!body.contains("this.title"));
        assert!(!body.contains("this.desc"));
        assert!(body.contains("_html_escape(title)"));
        assert!(body.contains("_html_escape(desc)"));
    }

    #[test]
    fn test_extract_page_layout() {
        assert_eq!(
            extract_page_layout("<page layout=\"main\"></page>\n<main>Hi</main>"),
            Some("main".to_string())
        );
        assert_eq!(
            extract_page_layout("<page layout='admin'></page>\n<div>X</div>"),
            Some("admin".to_string())
        );
        assert_eq!(extract_page_layout("<main>Hi</main>"), None);
    }

    #[test]
    fn test_parse_project_config_port() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_config_port");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("config.cln")).unwrap();
        writeln!(f, "config:").unwrap();
        writeln!(f, "\tserver:").unwrap();
        writeln!(f, "\t\tport = 3001").unwrap();
        let config = parse_project_config(&dir);
        assert_eq!(config.port, 3001);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_project_config_imports() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_config_imports");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("config.cln")).unwrap();
        writeln!(f, "config:").unwrap();
        writeln!(f, "\timports:").unwrap();
        writeln!(f, "\t\t\"app/server/helpers.cln\"").unwrap();
        writeln!(f, "\t\t\"app/server/utils.cln\"").unwrap();
        let config = parse_project_config(&dir);
        assert_eq!(config.imports.len(), 2);
        assert_eq!(config.imports[0], "app/server/helpers.cln");
        assert_eq!(config.imports[1], "app/server/utils.cln");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_import_paths_have_prefix() {
        // Bug 16: Import paths need ../../ prefix for dist/.generated/ location
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_import_prefix");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("config.cln")).unwrap();
        writeln!(f, "config:").unwrap();
        writeln!(f, "\timports:").unwrap();
        writeln!(f, "\t\t\"app/server/helpers.cln\"").unwrap();
        writeln!(f, "\troutes:").unwrap();
        writeln!(f, "\t\t\"app/server/api.cln\"").unwrap();

        let config = parse_project_config(&dir);
        let project = DiscoveredProject::default();
        let result = generate_imports(&project, &dir, &config).unwrap();
        assert!(
            result.contains("\"../../app/server/helpers.cln\""),
            "Import should have ../../ prefix: {}",
            result
        );
        assert!(
            result.contains("\"../../app/server/api.cln\""),
            "Route import should have ../../ prefix: {}",
            result
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_project_config_routes() {
        // Bug 19/20: Config should support routes: section
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_config_routes");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("config.cln")).unwrap();
        writeln!(f, "config:").unwrap();
        writeln!(f, "\timports:").unwrap();
        writeln!(f, "\t\t\"app/server/helpers.cln\"").unwrap();
        writeln!(f, "\troutes:").unwrap();
        writeln!(f, "\t\t\"app/server/api.cln\"").unwrap();
        writeln!(f, "\t\t\"app/server/errors_api.cln\"").unwrap();
        writeln!(f, "\t\t\"app/server/errors_pages.cln\"").unwrap();

        let config = parse_project_config(&dir);
        assert_eq!(config.imports.len(), 1);
        assert_eq!(config.imports[0], "app/server/helpers.cln");
        assert_eq!(config.routes.len(), 3);
        assert_eq!(config.routes[0], "app/server/api.cln");
        assert_eq!(config.routes[1], "app/server/errors_api.cln");
        assert_eq!(config.routes[2], "app/server/errors_pages.cln");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_page_data_block() {
        // Bug 18: Data block should be extracted from page's script tag
        let page = r#"<page layout="main"></page>
<main><h1>{{msg}}</h1></main>
<script type="text/clean">
    string msg = "Hello"
    string lang = "en"
</script>"#;
        let data = extract_page_data_block(page);
        assert!(
            data.contains("string msg = \"Hello\""),
            "Should extract msg: {}",
            data
        );
        assert!(
            data.contains("string lang = \"en\""),
            "Should extract lang: {}",
            data
        );
    }

    #[test]
    fn test_scan_route_files() {
        // Bug 20: Should find max handler index and extract route lines
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_scan_routes");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("app/server")).unwrap();

        // Create a fake route file
        let mut f = std::fs::File::create(dir.join("app/server/api.cln")).unwrap();
        writeln!(f, "functions:").unwrap();
        writeln!(f, "\tstring __route_handler_0()").unwrap();
        writeln!(f, "\t\treturn \"ok\"").unwrap();
        writeln!(f, "\tstring __route_handler_5()").unwrap();
        writeln!(f, "\t\treturn \"ok\"").unwrap();

        let routes = vec!["app/server/api.cln".to_string()];
        let (next_index, _lines) = scan_route_files(&routes, &dir);
        assert_eq!(next_index, 6, "Next index should be max(5) + 1 = 6");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_route_files_with_http_route_calls() {
        // Bug 20: Should extract _http_route() lines for start: block
        use std::io::Write;
        let dir = std::env::temp_dir().join("cleen_test_scan_route_calls");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("app/server")).unwrap();

        let mut f = std::fs::File::create(dir.join("app/server/api.cln")).unwrap();
        writeln!(f, "// route defs").unwrap();
        writeln!(f, "\ts = _http_route(\"GET\", \"/api/health\", 0)").unwrap();
        writeln!(f, "\ts = _http_route(\"GET\", \"/api/content\", 1)").unwrap();
        writeln!(f, "\ts = _http_route(\"GET\", \"/api/modules\", 5)").unwrap();

        let routes = vec!["app/server/api.cln".to_string()];
        let (next_index, lines) = scan_route_files(&routes, &dir);
        assert_eq!(next_index, 6, "Next index should be max(5) + 1 = 6");
        assert_eq!(lines.len(), 3, "Should have 3 route lines");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_route_handler_index() {
        assert_eq!(
            extract_route_handler_index("s = _http_route(\"GET\", \"/api/health\", 0)"),
            Some(0)
        );
        assert_eq!(
            extract_route_handler_index("s = _http_route(\"POST\", \"/api/v1/reports\", 13)"),
            Some(13)
        );
        assert_eq!(extract_route_handler_index("// not a route"), None);
    }

    #[test]
    fn test_extract_handler_def_index() {
        assert_eq!(
            extract_handler_def_index("string __route_handler_0()"),
            Some(0)
        );
        assert_eq!(
            extract_handler_def_index("\tstring __route_handler_21()"),
            Some(21)
        );
        assert_eq!(extract_handler_def_index("string foo()"), None);
    }

    #[test]
    fn test_component_tag_expansion_in_html() {
        // Bug 17: Component tags should be replaced with function calls
        let components = vec![Component {
            tag: "site-navbar".to_string(),
            class_name: "Navbar".to_string(),
            source_file: std::path::PathBuf::from("Navbar.cln"),
            hydration: "off".to_string(),
        }];
        let expanded = expand_component_tags("<site-navbar></site-navbar>", &components);
        assert!(
            expanded.contains("__component_Navbar_render()"),
            "Should replace tag with function call: {}",
            expanded
        );
        assert!(
            !expanded.contains("<site-navbar>"),
            "Should not contain literal tag: {}",
            expanded
        );
    }

    #[test]
    fn test_handler_offset_in_start_block() {
        // Bug 20: Framework handlers should start after imported handler indices
        let project = DiscoveredProject {
            pages: vec![PageRoute {
                method: "GET".to_string(),
                path: "/test".to_string(),
                source_file: std::path::PathBuf::from("test.html"),
                handler_index: 0,
                layout: None,
                auth: None,
                cache: None,
            }],
            ..Default::default()
        };
        let options = CodegenOptions::default();
        let imported_lines = vec![
            "s = _http_route(\"GET\", \"/api/health\", 0)".to_string(),
            "s = _http_route(\"GET\", \"/api/modules\", 5)".to_string(),
        ];
        let result =
            generate_start_function(&project, &options, 3001, 22, &imported_lines).unwrap();
        // Framework page route should use index 22
        assert!(
            result.contains("_http_route(\"GET\", \"/test\", 22)"),
            "Handler should start at offset 22: {}",
            result
        );
        // Imported routes should be included
        assert!(
            result.contains("_http_route(\"GET\", \"/api/health\", 0)"),
            "Imported routes should be included: {}",
            result
        );
        assert!(
            result.contains("_http_listen(3001)"),
            "Should use configured port: {}",
            result
        );
    }
}
