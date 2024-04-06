use sea_query::*;
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgQueryResult;
use std::env;
use uuid::Uuid;

enum Image {
    Table,
    Uuid,
    FileName,
    Owner,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ImageStruct {
    uuid: Uuid,
    file_name: String,
    owner: String,
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

pub async fn create_table() -> Result<PgQueryResult, sqlx::Error> {
    let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
        .await
        .unwrap();

    let sql = Table::create()
        .table(Image::Table)
        .if_not_exists()
        .col(ColumnDef::new(Image::Uuid).uuid().not_null().primary_key())
        .col(ColumnDef::new(Image::FileName).string().not_null())
        .col(ColumnDef::new(Image::Owner).string().not_null())
        .build(PostgresQueryBuilder);

    sqlx::query(&sql).execute(&pool).await
}

pub async fn insert_image_record(
    uuid: Uuid,
    file_name: &str,
    owner: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
        .await
        .unwrap();

    let (sql, values) = Query::insert()
        .into_table(Image::Table)
        .columns([Image::Uuid, Image::FileName, Image::Owner])
        .values_panic([uuid.into(), file_name.into(), owner.into()])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await
}

pub async fn get_row(uuid: Uuid) -> Result<ImageStruct, sqlx::Error> {
    let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
        .await
        .unwrap();

    let (sql, values) = Query::select()
        .columns([Image::Uuid, Image::FileName, Image::Owner])
        .from(Image::Table)
        .and_where(Expr::col(Image::Uuid).eq(uuid))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, ImageStruct, _>(&sql, values)
        .fetch_one(&pool)
        .await
}

pub async fn delete_image_record(uuid: Uuid) -> Result<PgQueryResult, sqlx::Error> {
    let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
        .await
        .unwrap();

    let (sql, values) = Query::delete()
        .from_table(Image::Table)
        .and_where(Expr::col(Image::Uuid).eq(uuid))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await
}

// pub async fn update_file_name(
//     uuid: Uuid,
//     new_file_name: &str,
// ) -> Result<PgQueryResult, sqlx::Error> {
//     let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
//         .await
//         .unwrap();

//     let (sql, values) = Query::update()
//         .table(Image::Table)
//         .value(Image::FileName, new_file_name)
//         .and_where(Expr::col(Image::Uuid).eq(uuid))
//         .build_sqlx(PostgresQueryBuilder);

//     sqlx::query_with(&sql, values).execute(&pool).await
// }
