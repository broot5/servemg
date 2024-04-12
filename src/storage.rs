use aws_sdk_s3::{
    config::Region,
    error::SdkError,
    operation::{
        create_bucket::{CreateBucketError, CreateBucketOutput},
        put_object::{PutObjectError, PutObjectOutput},
    },
    primitives::ByteStream,
    types::{BucketLocationConstraint, CreateBucketConfiguration},
    Client,
};
use bytes::{Bytes, BytesMut};

pub async fn get_client(
    s3_access_key_id: &str,
    s3_secret_access_key: &str,
    s3_endpoint_url: &str,
    s3_region: &str,
) -> Result<Client, aws_sdk_s3::Error> {
    let credentials = aws_sdk_s3::config::Credentials::new(
        s3_access_key_id,
        s3_secret_access_key,
        None,
        None,
        "loaded-from-custom-env",
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(s3_endpoint_url)
        .credentials_provider(credentials)
        .region(Region::new(s3_region.to_string()))
        .force_path_style(true)
        .build();

    Ok(aws_sdk_s3::Client::from_conf(s3_config))
}

pub async fn show_buckets(client: &Client, region: &str) -> Result<Vec<String>, aws_sdk_s3::Error> {
    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets();

    let mut result: Vec<String> = Vec::new();

    for bucket in buckets {
        let r = client
            .get_bucket_location()
            .bucket(bucket.name().unwrap_or_default())
            .send()
            .await?;

        if r.location_constraint().unwrap().as_ref() == region {
            //println!("{}", bucket.name().unwrap_or_default());
            result.push(bucket.name().unwrap_or_default().to_string())
        }
    }

    Ok(result)
}

pub async fn create_bucket(
    client: &Client,
    region: &str,
    bucket: &str,
) -> Result<CreateBucketOutput, SdkError<CreateBucketError>> {
    let constraint = BucketLocationConstraint::from(region);
    let cfg = CreateBucketConfiguration::builder()
        .location_constraint(constraint)
        .build();
    client
        .create_bucket()
        .create_bucket_configuration(cfg)
        .bucket(bucket)
        .send()
        .await
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
