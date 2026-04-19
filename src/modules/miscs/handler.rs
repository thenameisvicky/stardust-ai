use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn download_compose() -> impl IntoResponse {
    let compose_file = include_str!("../../../docker-compose.yml");

    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/yaml"),
        (
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"docker-compose.yml\"",
        ),
    ];

    (headers, compose_file)
}
