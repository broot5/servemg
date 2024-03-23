use axum::{extract::Path, routing::get, Router};
use base64ct::{Base64UrlUnpadded, Encoding};
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/images/:id", get(get_image));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_image(Path(id): Path<String>) {
    let hash = Base64UrlUnpadded::decode_vec(id);
}
