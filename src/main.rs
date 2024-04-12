mod db;
mod storage;

use aws_sdk_s3::Client;
use axum::{
    extract::{DefaultBodyLimit, Json, Multipart, Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use db::ImageStruct;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    s3_client: Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    dotenvy::dotenv().ok();

    let state = AppState {
        pool: db::create_pool(&env::var("DB_URL").expect("DB_URL environment variable not found"))
            .await
            .unwrap(),
        s3_client: storage::get_client(
            &env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID environment variable not found"),
            &env::var("S3_SECRET_ACCESS_KEY")
                .expect("S3_SECRET_ACCESS_KEY environment variable not found"),
            &env::var("S3_ENDPOINT_URL").expect("S3_ENDPOINT_URL environment variable not found"),
            &env::var("S3_REGION").expect("S3_REGION environment variable not found"),
        )
        .await
        .unwrap(),
    };

    db::create_table(&state.pool).await.unwrap();

    if !storage::show_buckets(
        &state.s3_client,
        &env::var("S3_REGION").expect("S3_REGION environment variable not found"),
    )
    .await
    .unwrap()
    .iter()
    .any(|x| x == "image")
    {
        storage::create_bucket(
            &state.s3_client,
            &env::var("S3_REGION").expect("S3_REGION environment variable not found"),
            "image",
        )
        .await
        .unwrap();
    }

    let app = Router::new()
        .route("/images", post(upload_image))
        .route(
            "/images/:uuid",
            get(get_image).delete(delete_image).patch(patch_image),
        )
        .route("/images/:uuid/view", get(view_image))
        .with_state(state)
        .layer(DefaultBodyLimit::max(52428800)); // 50MB

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn upload_image(State(state): State<AppState>, mut multipart: Multipart) -> Response {
    // 이미지 읽어서 storage에 저장 후 db에 저장
    match multipart.next_field().await.unwrap() {
        Some(field) => {
            // let name = field.name().unwrap_or_default().to_string();
            let file_name = field.file_name().unwrap_or_default().to_string();
            // let content_type = field.content_type().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            // Check if content_type is image

            let record = ImageStruct {
                uuid: Uuid::new_v4(),
                file_name,
                owner: "anon".to_string(),
            };

            storage::upload_object(
                &state.s3_client,
                "image",
                data,
                &record.uuid.as_hyphenated().to_string(),
            )
            .await
            .unwrap();

            db::insert_image_record(&state.pool, &record).await.unwrap();

            (StatusCode::CREATED, Json(&record)).into_response()
        }
        None => (StatusCode::BAD_REQUEST, "No image provided").into_response(),
    }
}

async fn get_image(State(state): State<AppState>, Path(uuid): Path<String>) -> Response {
    // uuid로 storage 에서 바로 불러오기
    match db::get_image_record(&state.pool, Uuid::parse_str(&uuid).unwrap_or_default()).await {
        Ok(row) => {
            let content_type = mime_guess::from_path(std::path::Path::new(&row.file_name))
                .first_raw()
                .unwrap();

            let headers = [
                (header::CONTENT_TYPE, content_type.to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!(r#"attachment; filename="{}""#, &row.file_name),
                ),
            ];

            let body = storage::get_object(
                &state.s3_client,
                "image",
                &row.uuid.as_hyphenated().to_string(),
            )
            .await
            .unwrap();

            (StatusCode::OK, headers, body).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!("Image with UUID {} not found", uuid),
        )
            .into_response(),
    }
}

async fn delete_image(State(state): State<AppState>, Path(uuid): Path<String>) -> Response {
    // 해당 uuid 가진 row storage에서 삭제하고 db에서 삭제
    storage::remove_object(&state.s3_client, "image", &uuid)
        .await
        .unwrap();

    db::delete_image_record(&state.pool, Uuid::parse_str(&uuid).unwrap_or_default())
        .await
        .unwrap();

    StatusCode::OK.into_response()
}

async fn patch_image(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // 해당 uuid 가진 row의 file_name column 변경
    // 해당 uuid 가진 row의 owner column 변경
    let new_file_name = payload
        .get("file_name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let new_owner = payload
        .get("owner")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    if new_file_name.is_none() && new_owner.is_none() {
        return (StatusCode::BAD_REQUEST, "No valid data provided for update").into_response();
    }

    db::update_image_record(
        &state.pool,
        Uuid::parse_str(&uuid).unwrap_or_default(),
        new_file_name,
        new_owner,
    )
    .await
    .unwrap();

    (StatusCode::OK, "Image record updated successfully").into_response()
}

async fn view_image(Path(uuid): Path<String>) -> Response {
    Html(format!(r#"<img src="/images/{}" alt="image">"#, uuid)).into_response()
}
