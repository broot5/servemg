mod db;
mod storage;

use axum::{
    extract::{Multipart, Path},
    routing::{get, patch, post},
    Router,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");

    db::create_table().await;

    let app = Router::new()
        .route("/images", post(upload_image))
        .route("/images/:id", get(get_image).delete(delete_image))
        .route("/images/:id/name", patch(rename_image))
        .route("/images/:id/owner", patch(transfer_image));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn upload_image(mut multipart: Multipart) {
    // 이미지 읽어서 storage에 저장 후 db에 저장
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );

        let client = storage::get_client().await.unwrap();
        storage::upload_object(&client, "image", data, &file_name)
            .await
            .unwrap();
    }
}

async fn get_image(Path(id): Path<String>) {
    // id로 storage 에서 바로 불러오기
}

async fn delete_image(Path(id): Path<String>) {
    // 해당 id 가진 row storage에서 삭제하고 db에서 삭제
}

async fn rename_image(Path(id): Path<String>) {
    // 해당 id 가진 row의 name column 변경
}

async fn transfer_image(Path(id): Path<String>) {
    // 해당 id 가진 row의 owner column 변경
}
