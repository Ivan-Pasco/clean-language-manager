//! File discovery module for automatic route and component detection
//!
//! Scans project directories to discover:
//! - Pages (app/ui/pages/) -> HTML routes
//! - Components (app/ui/components/) -> Custom elements
//! - Layouts (app/ui/layouts/) -> Page wrappers
//! - API routes (app/server/api/) -> JSON endpoints
//! - Models (app/server/models/) -> Database schemas
//! - Middleware (app/server/middleware/) -> Request filters

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// A discovered HTML page route
#[derive(Debug, Clone)]
pub struct PageRoute {
    /// HTTP method (always "GET" for pages)
    pub method: String,
    /// URL path (e.g., "/blog/:slug")
    pub path: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Handler index for route registration
    pub handler_index: usize,
    /// Optional layout name
    pub layout: Option<String>,
    /// Optional auth requirement
    pub auth: Option<String>,
    /// Optional cache settings
    pub cache: Option<String>,
}

/// A discovered API route
#[derive(Debug, Clone)]
pub struct ApiRoute {
    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: String,
    /// URL path (e.g., "/api/users/:id")
    pub path: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Handler index for route registration
    pub handler_index: usize,
    /// Middleware to apply
    pub middleware: Vec<String>,
}

/// A discovered UI component
#[derive(Debug, Clone)]
pub struct Component {
    /// HTML tag name (e.g., "user-card")
    pub tag: String,
    /// Class name (e.g., "UserCard")
    pub class_name: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Hydration mode ("off", "on", "visible", "idle")
    pub hydration: String,
}

/// A discovered layout
#[derive(Debug, Clone)]
pub struct Layout {
    /// Layout name (e.g., "main", "admin")
    pub name: String,
    /// Source file path
    pub source_file: PathBuf,
}

/// A discovered database model
#[derive(Debug, Clone)]
pub struct Model {
    /// Model name (e.g., "User")
    pub name: String,
    /// Table name (e.g., "users")
    pub table: String,
    /// Source file path
    pub source_file: PathBuf,
}

/// A discovered middleware
#[derive(Debug, Clone)]
pub struct Middleware {
    /// Middleware name (e.g., "auth")
    pub name: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Route patterns to apply to
    pub applies_to: Vec<String>,
}

/// A discovered library module
#[derive(Debug, Clone)]
pub struct LibModule {
    /// Module name
    pub name: String,
    /// Source file path
    pub source_file: PathBuf,
}

/// Complete discovered project structure
#[derive(Debug, Default)]
pub struct DiscoveredProject {
    // UI
    pub pages: Vec<PageRoute>,
    pub components: Vec<Component>,
    pub layouts: Vec<Layout>,
    // Server
    pub api_routes: Vec<ApiRoute>,
    pub models: Vec<Model>,
    pub middleware: Vec<Middleware>,
    // Shared
    pub lib_modules: Vec<LibModule>,
    // Static files directory (if exists)
    pub public_dir: Option<PathBuf>,
}

impl DiscoveredProject {
    /// Check if project has any discovered content
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
            && self.components.is_empty()
            && self.layouts.is_empty()
            && self.api_routes.is_empty()
            && self.models.is_empty()
            && self.middleware.is_empty()
            && self.lib_modules.is_empty()
    }

    /// Get total count of discovered items
    pub fn total_count(&self) -> usize {
        self.pages.len()
            + self.components.len()
            + self.layouts.len()
            + self.api_routes.len()
            + self.models.len()
            + self.middleware.len()
            + self.lib_modules.len()
    }
}

/// Discover all project files and return structured discovery result
pub fn discover_project(project_dir: &Path) -> Result<DiscoveredProject> {
    let mut project = DiscoveredProject::default();
    let app_dir = project_dir.join("app");

    if !app_dir.exists() {
        // Try root-level folders for simpler projects
        let ui_dir = project_dir.join("ui");
        let server_dir = project_dir.join("server");

        if ui_dir.exists() || server_dir.exists() {
            discover_ui(&ui_dir, &mut project)?;
            discover_server(&server_dir, &mut project)?;
            discover_shared(&project_dir.join("shared"), &mut project)?;
        }

        return Ok(project);
    }

    // Standard app/ structure
    discover_ui(&app_dir.join("ui"), &mut project)?;
    discover_server(&app_dir.join("server"), &mut project)?;
    discover_shared(&app_dir.join("shared"), &mut project)?;

    Ok(project)
}

/// Discover UI components: pages, components, layouts, public
fn discover_ui(ui_dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !ui_dir.exists() {
        return Ok(());
    }

    // Discover pages
    let pages_dir = ui_dir.join("pages");
    if pages_dir.exists() {
        discover_pages(&pages_dir, &pages_dir, project)?;
    }

    // Discover components
    let components_dir = ui_dir.join("components");
    if components_dir.exists() {
        discover_components(&components_dir, &components_dir, project)?;
    }

    // Discover layouts
    let layouts_dir = ui_dir.join("layouts");
    if layouts_dir.exists() {
        discover_layouts(&layouts_dir, project)?;
    }

    // Check for public directory
    let public_dir = ui_dir.join("public");
    if public_dir.exists() {
        project.public_dir = Some(public_dir);
    }

    Ok(())
}

/// Discover server components: api routes, models, middleware
fn discover_server(server_dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !server_dir.exists() {
        return Ok(());
    }

    // Discover API routes
    let api_dir = server_dir.join("api");
    if api_dir.exists() {
        discover_api_routes(&api_dir, &api_dir, project)?;
    }

    // Discover models
    let models_dir = server_dir.join("models");
    if models_dir.exists() {
        discover_models(&models_dir, project)?;
    }

    // Discover middleware
    let middleware_dir = server_dir.join("middleware");
    if middleware_dir.exists() {
        discover_middleware(&middleware_dir, project)?;
    }

    Ok(())
}

/// Discover shared library modules
fn discover_shared(shared_dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !shared_dir.exists() {
        return Ok(());
    }

    let lib_dir = shared_dir.join("lib");
    if lib_dir.exists() {
        discover_lib_modules(&lib_dir, project)?;
    }

    Ok(())
}

/// Recursively discover page routes
fn discover_pages(dir: &Path, base_dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read pages directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            discover_pages(&path, base_dir, project)?;
        } else if is_page_file(&path) {
            let route_path = file_to_route_path(&path, base_dir);
            let handler_index = project.pages.len();

            project.pages.push(PageRoute {
                method: "GET".to_string(),
                path: route_path,
                source_file: path,
                handler_index,
                layout: None,
                auth: None,
                cache: None,
            });
        }
    }

    Ok(())
}

/// Recursively discover components
fn discover_components(
    dir: &Path,
    _base_dir: &Path,
    project: &mut DiscoveredProject,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read components directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            discover_components(&path, _base_dir, project)?;
        } else if is_cln_file(&path) {
            let class_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let tag = class_name_to_tag(&class_name);

            project.components.push(Component {
                tag,
                class_name,
                source_file: path,
                hydration: "off".to_string(),
            });
        }
    }

    Ok(())
}

/// Discover layouts
fn discover_layouts(dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read layouts directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && (is_cln_file(&path) || is_page_file(&path)) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_end_matches(".html"))
                .unwrap_or("unknown")
                .to_string();

            project.layouts.push(Layout {
                name,
                source_file: path,
            });
        }
    }

    Ok(())
}

/// Recursively discover API routes
fn discover_api_routes(dir: &Path, base_dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read API directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            discover_api_routes(&path, base_dir, project)?;
        } else if is_cln_file(&path) {
            let route_path = file_to_api_route_path(&path, base_dir);
            let handler_index = project.api_routes.len();

            // Default to GET, but the file may contain multiple methods
            project.api_routes.push(ApiRoute {
                method: "GET".to_string(),
                path: route_path,
                source_file: path,
                handler_index,
                middleware: Vec::new(),
            });
        }
    }

    Ok(())
}

/// Discover database models
fn discover_models(dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read models directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_cln_file(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            // Convert PascalCase to snake_case for table name
            let table = pascal_to_snake(&name);

            project.models.push(Model {
                name,
                table,
                source_file: path,
            });
        }
    }

    Ok(())
}

/// Discover middleware
fn discover_middleware(dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read middleware directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_cln_file(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            project.middleware.push(Middleware {
                name,
                source_file: path,
                applies_to: Vec::new(),
            });
        }
    }

    Ok(())
}

/// Discover library modules
fn discover_lib_modules(dir: &Path, project: &mut DiscoveredProject) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context("Failed to read lib directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_cln_file(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            project.lib_modules.push(LibModule {
                name,
                source_file: path,
            });
        }
    }

    Ok(())
}

// Helper functions

/// Check if file is a .cln file
fn is_cln_file(path: &Path) -> bool {
    path.extension().map(|ext| ext == "cln").unwrap_or(false)
}

/// Check if file is a page file (.html.cln or .html)
fn is_page_file(path: &Path) -> bool {
    path.to_str()
        .map(|s| s.ends_with(".html.cln") || s.ends_with(".html"))
        .unwrap_or(false)
}

/// Convert file path to URL route path
/// Examples:
///   pages/index.html.cln -> /
///   pages/about.html.cln -> /about
///   pages/blog/index.html.cln -> /blog
///   pages/blog/[slug].html.cln -> /blog/:slug
fn file_to_route_path(file_path: &Path, base_dir: &Path) -> String {
    let relative = file_path.strip_prefix(base_dir).unwrap_or(file_path);

    let path_str = relative.to_string_lossy();

    // Remove .html.cln or .html extension
    let path_str = path_str
        .trim_end_matches(".html.cln")
        .trim_end_matches(".html");

    // Handle index files
    let path_str = if path_str == "index" || path_str.is_empty() {
        "/".to_string()
    } else if path_str.ends_with("/index") {
        format!("/{}", path_str.trim_end_matches("/index"))
    } else {
        format!("/{}", path_str)
    };

    // Convert [param] to :param
    convert_params(&path_str)
}

/// Convert file path to API route path (with /api/ prefix)
fn file_to_api_route_path(file_path: &Path, base_dir: &Path) -> String {
    let relative = file_path.strip_prefix(base_dir).unwrap_or(file_path);

    let path_str = relative.to_string_lossy();

    // Remove .cln extension
    let path_str = path_str.trim_end_matches(".cln");

    // Handle index files
    let path_str = if path_str == "index" || path_str.is_empty() {
        "/api".to_string()
    } else if path_str.ends_with("/index") {
        format!("/api/{}", path_str.trim_end_matches("/index"))
    } else {
        format!("/api/{}", path_str)
    };

    // Convert [param] to :param
    convert_params(&path_str)
}

/// Convert [param] syntax to :param
fn convert_params(path: &str) -> String {
    let mut result = String::new();
    let mut in_bracket = false;
    let mut param = String::new();

    for ch in path.chars() {
        match ch {
            '[' => {
                in_bracket = true;
                param.clear();
            }
            ']' => {
                in_bracket = false;
                result.push(':');
                result.push_str(&param);
            }
            _ if in_bracket => {
                param.push(ch);
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Convert PascalCase class name to kebab-case tag name
/// Examples:
///   Header -> app-header
///   UserCard -> user-card
///   BlogPostPreview -> blog-post-preview
fn class_name_to_tag(class_name: &str) -> String {
    let mut result = String::new();

    for ch in class_name.chars() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('-');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    // Add app- prefix for single-word components to ensure valid custom element name
    if !result.contains('-') {
        result = format!("app-{}", result);
    }

    result
}

/// Convert PascalCase to snake_case
/// Examples:
///   User -> users
///   BlogPost -> blog_posts
fn pascal_to_snake(name: &str) -> String {
    let mut result = String::new();

    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    // Pluralize (simple version - just add 's')
    if !result.ends_with('s') {
        result.push('s');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_to_route_path() {
        let base = Path::new("/app/ui/pages");

        // .html.cln extension
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/index.html.cln"), base),
            "/"
        );
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/about.html.cln"), base),
            "/about"
        );
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/blog/index.html.cln"), base),
            "/blog"
        );
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/blog/[slug].html.cln"), base),
            "/blog/:slug"
        );

        // .html extension
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/index.html"), base),
            "/"
        );
        assert_eq!(
            file_to_route_path(Path::new("/app/ui/pages/about.html"), base),
            "/about"
        );
    }

    #[test]
    fn test_is_page_file() {
        assert!(is_page_file(Path::new("test.html.cln")));
        assert!(is_page_file(Path::new("test.html")));
        assert!(!is_page_file(Path::new("test.cln")));
        assert!(!is_page_file(Path::new("test.txt")));
    }

    #[test]
    fn test_class_name_to_tag() {
        assert_eq!(class_name_to_tag("Header"), "app-header");
        assert_eq!(class_name_to_tag("UserCard"), "user-card");
        assert_eq!(class_name_to_tag("BlogPostPreview"), "blog-post-preview");
    }

    #[test]
    fn test_pascal_to_snake() {
        assert_eq!(pascal_to_snake("User"), "users");
        assert_eq!(pascal_to_snake("BlogPost"), "blog_posts");
        assert_eq!(pascal_to_snake("Articles"), "articles");
    }

    #[test]
    fn test_convert_params() {
        assert_eq!(convert_params("/blog/[slug]"), "/blog/:slug");
        assert_eq!(convert_params("/users/[id]/posts"), "/users/:id/posts");
        assert_eq!(convert_params("/api/articles/[id]"), "/api/articles/:id");
    }
}
