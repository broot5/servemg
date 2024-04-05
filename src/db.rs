use sea_query::*;
use std::env;

pub enum Image {
    Table,
    Id,
    Name,
    Owner,
}

impl Iden for Image {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "image",
                Self::Id => "id",
                Self::Name => "name",
                Self::Owner => "owner",
            }
        )
        .unwrap();
    }
}

pub async fn create_table() {
    let pool = sqlx::PgPool::connect(&env::var("DB_URL").unwrap())
        .await
        .unwrap();

    let table = Table::create()
        .table(Image::Table)
        .if_not_exists()
        .col(ColumnDef::new(Image::Id).uuid().not_null().primary_key())
        .col(ColumnDef::new(Image::Name).string().not_null())
        .col(ColumnDef::new(Image::Owner).string().not_null())
        .build(PostgresQueryBuilder);
    let result = sqlx::query(&table).execute(&pool).await;
    println!("Create table image: {result:?}");
}
