use sea_query::*;
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use std::env;
use uuid::Uuid;

enum Image {
    Table,
    Uuid,
    FileName,
    Owner,
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug)]
pub struct ImageStruct {
    pub uuid: Uuid,
    pub file_name: String,
    pub owner: String,
}

impl Iden for Image {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "image",
                Self::Uuid => "uuid",
                Self::FileName => "file_name",
                Self::Owner => "owner",
            }
        )
        .unwrap();
    }
}

pub async fn create_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let db_url = env::var("DB_URL").expect("DB_URL environment variable not found");
    sqlx::PgPool::connect(&db_url).await
}

pub async fn create_table(pool: &Pool<Postgres>) -> Result<PgQueryResult, sqlx::Error> {
    let sql = Table::create()
        .table(Image::Table)
        .if_not_exists()
        .col(ColumnDef::new(Image::Uuid).uuid().not_null().primary_key())
        .col(ColumnDef::new(Image::FileName).string().not_null())
        .col(ColumnDef::new(Image::Owner).string().not_null())
        .build(PostgresQueryBuilder);

    sqlx::query(&sql).execute(pool).await
}

pub async fn insert_image_record(
    pool: &Pool<Postgres>,
    image_struct: &ImageStruct,
) -> Result<PgQueryResult, sqlx::Error> {
    let (sql, values) = Query::insert()
        .into_table(Image::Table)
        .columns([Image::Uuid, Image::FileName, Image::Owner])
        .values_panic([
            image_struct.uuid.into(),
            image_struct.file_name.clone().into(),
            image_struct.owner.clone().into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await
}

pub async fn get_image_record(
    pool: &Pool<Postgres>,
    uuid: Uuid,
) -> Result<ImageStruct, sqlx::Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(Image::Table)
        .and_where(Expr::col(Image::Uuid).eq(uuid))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, ImageStruct, _>(&sql, values)
        .fetch_one(pool)
        .await
}

pub async fn delete_image_record(
    pool: &Pool<Postgres>,
    uuid: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    let (sql, values) = Query::delete()
        .from_table(Image::Table)
        .and_where(Expr::col(Image::Uuid).eq(uuid))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await
}

pub async fn update_image_record(
    pool: &Pool<Postgres>,
    uuid: Uuid,
    new_file_name: Option<&str>,
    new_owner: Option<&str>,
) -> Result<PgQueryResult, sqlx::Error> {
    let mut query = Query::update()
        .table(Image::Table)
        .and_where(Expr::col(Image::Uuid).eq(uuid))
        .to_owned();

    if let Some(file_name) = new_file_name {
        query = query.value(Image::FileName, file_name).to_owned();
    }

    if let Some(owner) = new_owner {
        query = query.value(Image::Owner, owner).to_owned();
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await
}
