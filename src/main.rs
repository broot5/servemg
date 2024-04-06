mod db;
mod storage;

use axum::{
    extract::{DefaultBodyLimit, Json, Multipart, Path},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use db::ImageStruct;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");

    db::create_table().await.unwrap();

    let app = Router::new()
        .route("/images", post(upload_image))
        .route(
            "/images/:uuid",
            get(get_image).delete(delete_image).patch(patch_image),
        )
        .route("/images/:uuid/view", get(view_image))
        .layer(DefaultBodyLimit::max(52428800)); // 50MB

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn upload_image(mut multipart: Multipart) -> impl IntoResponse {
    // 이미지 읽어서 storage에 저장 후 db에 저장
    match multipart.next_field().await.unwrap() {
        Some(field) => {
            let name = field.name().unwrap_or_default().to_string();
            let file_name = field.file_name().unwrap_or_default().to_string();
            let content_type = field.content_type().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            println!(
                "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
                data.len()
            );

            // Check if content_type is image

            // let file_extension = std::path::Path::new(&file_name)
            //     .extension()
            //     .unwrap()
            //     .to_str()
            //     .unwrap();

            // match file_extension {
            //     "jpg" => {}
            //     "png" => {}
            //     "webp" => {}
            //     _ => return StatusCode::UNSUPPORTED_MEDIA_TYPE,
            // }

            let row = ImageStruct {
                uuid: Uuid::new_v4(),
                file_name,
                owner: "anon".to_string(),
            };

            let client = storage::get_client().await.unwrap();
            storage::upload_object(
                &client,
                "image",
                data,
                &row.uuid.as_hyphenated().to_string(),
            )
            .await
            .unwrap();

            db::insert_image_record(&row).await.unwrap();

            (StatusCode::CREATED, Json(row))
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(ImageStruct {
                uuid: Uuid::nil(),
                file_name: String::new(),
                owner: String::new(),
            }),
        ),
    }
}

async fn get_image(Path(uuid): Path<String>) -> impl IntoResponse {
    // uuid로 storage 에서 바로 불러오기
    let row = db::get_image_record(Uuid::parse_str(&uuid).unwrap())
        .await
        .unwrap();

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

    let client = storage::get_client().await.unwrap();
    let body = storage::get_object(&client, "image", &row.uuid.as_hyphenated().to_string())
        .await
        .unwrap();

    (headers, body)
}

async fn delete_image(Path(uuid): Path<String>) -> impl IntoResponse {
    // 해당 uuid 가진 row storage에서 삭제하고 db에서 삭제
    let client = storage::get_client().await.unwrap();
    storage::remove_object(&client, "image", &uuid)
        .await
        .unwrap();

    db::delete_image_record(Uuid::parse_str(&uuid).unwrap())
        .await
        .unwrap();

    StatusCode::OK
}

async fn patch_image(Path(uuid): Path<String>, Json(payload): Json<serde_json::Value>) {
    // 해당 uuid 가진 row의 file_name column 변경
    // 해당 uuid 가진 row의 owner column 변경
    let new_file_name = &payload["file_name"];
    let new_owner = &payload["owner"];

    if new_file_name.is_string() {
        db::update_image_file_name(Uuid::parse_str(&uuid).unwrap(), &new_file_name.to_string())
            .await
            .unwrap();
    }

    if new_owner.is_string() {
        db::update_image_owner(Uuid::parse_str(&uuid).unwrap(), &new_owner.to_string())
            .await
            .unwrap();
    }
}

async fn view_image(Path(uuid): Path<String>) -> impl IntoResponse {
    Html(format!(r#"<img src="/images/{}" alt="image">"#, uuid))
}
