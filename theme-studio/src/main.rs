use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use cybercore::theme::{Appearance, ThemeCatalog, ThemeDocument};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;

const PAGE: &str = include_str!("../static/index.html");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/themes", get(list_themes).post(save_theme))
        .route("/api/themes/:id", delete(delete_theme))
        .route("/api/themes/validate", post(validate_theme))
        .route("/api/active/:id", post(select_theme))
        .route("/api/appearance/:mode", post(select_appearance))
        .route("/api/css/:id", get(theme_css))
        .layer(middleware::from_fn(local_host_only));

    let bind = std::env::var("CYBERCORE_THEME_STUDIO_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8761".to_string());
    let address: SocketAddr = bind.parse().expect("invalid CYBERCORE_THEME_STUDIO_BIND");
    if !address.ip().is_loopback() {
        eprintln!("Cybercore Theme Studio only accepts loopback bind addresses");
        std::process::exit(2);
    }
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("could not bind Cybercore Theme Studio");
    println!("Cybercore Theme Studio: http://{address}");
    axum::serve(listener, app).await.expect("server failed");
}

async fn local_host_only(request: Request<Body>, next: Next) -> Response {
    let allowed = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(is_local_host)
        .unwrap_or(false);
    if !allowed {
        return (StatusCode::FORBIDDEN, "local host required").into_response();
    }
    next.run(request).await
}

fn is_local_host(value: &str) -> bool {
    let Ok(authority) = value.parse::<axum::http::uri::Authority>() else {
        return false;
    };
    let host = authority.host().to_ascii_lowercase();
    matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]")
        || host.ends_with(".localhost")
}

async fn index() -> Html<&'static str> {
    Html(PAGE)
}

fn catalog_error(error: impl ToString) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": error.to_string()})),
    )
        .into_response()
}

async fn list_themes() -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let themes: Vec<Value> = catalog
        .iter()
        .map(|(id, entry)| {
            json!({
                "id": id,
                "builtin": entry.builtin,
                "document": entry.document,
            })
        })
        .collect();
    Json(json!({
        "active": catalog.active_id(),
        "appearance": catalog.active_appearance(),
        "themes": themes,
    }))
    .into_response()
}

async fn save_theme(Json(document): Json<ThemeDocument>) -> Response {
    let id = document.metadata.id.clone();
    let name = document.metadata.name.clone();
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    if let Err(error) = catalog.save(document) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response();
    }
    if let Err(error) = catalog.select(&id) {
        return catalog_error(error);
    }
    (StatusCode::CREATED, Json(json!({"id": id, "name": name}))).into_response()
}

async fn validate_theme(Json(document): Json<ThemeDocument>) -> Response {
    if let Err(error) = document.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response();
    }
    let light = document
        .variants
        .contains_key(&Appearance::Light)
        .then(|| document.contrast_report(Appearance::Light));
    Json(json!({
        "valid": true,
        "contrast": document.contrast_report(Appearance::Dark),
        "light_contrast": light,
    }))
    .into_response()
}

async fn select_theme(Path(id): Path<String>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.select(&id) {
        Ok(()) => Json(json!({"active": id})).into_response(),
        Err(error) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn select_appearance(Path(mode): Path<String>) -> Response {
    let appearance = match mode.as_str() {
        "dark" => Appearance::Dark,
        "light" => Appearance::Light,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "mode must be dark or light"})),
            )
                .into_response()
        }
    };
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.set_appearance(appearance) {
        Ok(()) => Json(json!({"appearance": appearance})).into_response(),
        Err(error) => catalog_error(error),
    }
}

async fn delete_theme(Path(id): Path<String>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.remove_custom(&id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct CssQuery {
    appearance: Option<Appearance>,
}

async fn theme_css(Path(id): Path<String>, Query(query): Query<CssQuery>) -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let Some(entry) = catalog.get(&id) else {
        return (StatusCode::NOT_FOUND, "unknown theme").into_response();
    };
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        entry
            .document
            .to_css(query.appearance.unwrap_or(catalog.active_appearance())),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::is_local_host;

    #[test]
    fn permits_loopback_hostnames_and_ports() {
        for host in [
            "localhost",
            "127.0.0.1:8761",
            "[::1]:8761",
            "cybercore-tech.localhost:8762",
        ] {
            assert!(is_local_host(host), "expected {host} to be local");
        }
    }

    #[test]
    fn rejects_non_loopback_and_suffix_spoofed_hosts() {
        for host in ["cybercore-tech.localhost.example.com", "example.com:8761"] {
            assert!(!is_local_host(host), "expected {host} to be rejected");
        }
    }
}
