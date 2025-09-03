//! Embedded UI assets for the FHIRPath server
//!
//! This module handles serving the embedded SolidJS web interface
//! that's built and included in the binary at compile time.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use include_dir::{Dir, include_dir};
use mime_guess::from_path;

// Include UI assets if they exist, otherwise include an empty directory
// This prevents compilation failures when building from crates.io or docs.rs
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../dist/ui");

/// Serve embedded UI assets with proper MIME types and SPA support
pub async fn serve_embedded_assets(
    Path(path): Path<String>,
) -> Result<Response<Body>, (StatusCode, String)> {
    // Handle root path and empty path
    let asset_path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.as_str()
    };

    match UI_ASSETS.get_file(asset_path) {
        Some(file) => {
            let mime_type = from_path(asset_path).first_or_octet_stream();
            let contents = file.contents();

            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type.as_ref());

            // Add cache headers for static assets
            if !asset_path.ends_with(".html") {
                response =
                    response.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
            }

            Ok(response.body(Body::from(contents)).unwrap())
        }
        None => {
            // For SPA routing - serve index.html for paths that don't match assets
            if !asset_path.contains('.') || asset_path.ends_with(".html") {
                match UI_ASSETS.get_file("index.html") {
                    Some(index_file) => {
                        let contents = index_file.contents();

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                            .body(Body::from(contents))
                            .unwrap())
                    }
                    None => Err((
                        StatusCode::NOT_FOUND,
                        "UI not available - index.html not found".to_string(),
                    )),
                }
            } else {
                Err((StatusCode::NOT_FOUND, "Asset not found".to_string()))
            }
        }
    }
}

/// Serve the root UI page
pub async fn serve_ui_root() -> Result<Response<Body>, (StatusCode, String)> {
    match UI_ASSETS.get_file("index.html") {
        Some(file) => {
            let contents = file.contents();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(contents))
                .unwrap())
        }
        None => Err((
            StatusCode::NOT_FOUND,
            "UI not available - run 'cd ui && pnpm install && pnpm build' to build the web interface".to_string(),
        )),
    }
}

/// Check if UI assets are available
pub fn ui_assets_available() -> bool {
    UI_ASSETS.get_file("index.html").is_some()
}

/// Get asset count for diagnostics
pub fn asset_count() -> usize {
    count_files(&UI_ASSETS)
}

fn count_files(dir: &Dir) -> usize {
    let mut count = 0;
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::File(_) => count += 1,
            include_dir::DirEntry::Dir(subdir) => count += count_files(subdir),
        }
    }
    count
}
