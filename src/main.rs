mod db;
mod storage;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    routing::{get, patch, post},
    Router,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");

    db::create_table().await.unwrap();

    let app = Router::new()
        .route("/images", post(upload_image))
        .route("/images/:uuid", get(get_image).delete(delete_image))
        .route("/images/:uuid/name", patch(rename_image))
        .route("/images/:uuid/owner", patch(transfer_image));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn upload_image(mut multipart: Multipart) -> (StatusCode, String) {
    // 이미지 읽어서 storage에 저장 후 db에 저장
    while let Some(field) = multipart.next_field().await.unwrap() {
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

        let uuid = Uuid::new_v4();

        let client = storage::get_client().await.unwrap();
        storage::upload_object(&client, "image", data, &uuid.as_hyphenated().to_string())
            .await
            .unwrap();

        db::insert_image_record(uuid, &file_name, "anon")
            .await
            .unwrap();

        return (
            StatusCode::CREATED,
            "Succesfully uploaded image".to_string(),
        );
    }
    return (StatusCode::BAD_REQUEST, "BAD REQUEST".to_string());
}

async fn get_image(Path(uuid): Path<String>) {
    // uuid로 storage 에서 바로 불러오기
    //let file_name =

    //mime_guess::from_path()
    let row = db::get_row(Uuid::parse_str(&uuid).unwrap()).await.unwrap();
}

async fn delete_image(Path(uuid): Path<String>) -> StatusCode {
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

async fn rename_image(Path(uuid): Path<String>) {
    // 해당 uuid 가진 row의 name column 변경
    //db::update_file_name(uuid, new_file_name)
}

async fn transfer_image(Path(uuid): Path<String>) {
    // 해당 id 가진 row의 owner column 변경
}
