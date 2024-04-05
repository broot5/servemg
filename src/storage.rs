use aws_sdk_s3::{
    config::Region,
    error::SdkError,
    operation::put_object::{PutObjectError, PutObjectOutput},
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;
use std::env;

pub async fn get_client() -> Result<Client, aws_sdk_s3::Error> {
    let credentials = aws_sdk_s3::config::Credentials::new(
        env::var("S3_ACCESS_KEY_ID").unwrap(),
        env::var("S3_SECRET_ACCESS_KEY").unwrap(),
        None,
        None,
        "loaded-from-custom-env",
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(env::var("S3_ENDPOINT_URL").unwrap())
        .credentials_provider(credentials)
        .region(Region::new(env::var("S3_REGION").unwrap()))
        .build();

    Ok(aws_sdk_s3::Client::from_conf(s3_config))
}

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    bytes: Bytes,
    key: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let body = ByteStream::from(bytes);
    client
        .put_object()
        .bucket(bucket_name)
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
