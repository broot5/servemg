use aws_sdk_s3::{
    config::Region,
    error::SdkError,
    operation::put_object::{PutObjectError, PutObjectOutput},
    primitives::ByteStream,
    Client,
};
use bytes::{Bytes, BytesMut};
use std::env;

pub async fn get_client() -> Result<Client, aws_sdk_s3::Error> {
    let credentials = aws_sdk_s3::config::Credentials::new(
        env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID environment variable not found"),
        env::var("S3_SECRET_ACCESS_KEY")
            .expect("S3_SECRET_ACCESS_KEY environment variable not found"),
        None,
        None,
        "loaded-from-custom-env",
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(
            env::var("S3_ENDPOINT_URL").expect("S3_ENDPOINT_URL environment variable not found"),
        )
        .credentials_provider(credentials)
        .region(Region::new(
            env::var("S3_REGION").expect("S3_REGION environment variable not found"),
        ))
        .build();

    Ok(aws_sdk_s3::Client::from_conf(s3_config))
}

pub async fn upload_object(
    client: &Client,
    bucket: &str,
    bytes: Bytes,
    key: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let body = ByteStream::from(bytes);
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await
}

pub async fn remove_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<(), aws_sdk_s3::Error> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    Ok(())
}

pub async fn get_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<Bytes, aws_sdk_s3::Error> {
    let mut object = client.get_object().bucket(bucket).key(key).send().await?;

    let mut bytes = BytesMut::new();
    while let Some(chunk) = object.body.try_next().await.unwrap() {
        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes.freeze())
}
