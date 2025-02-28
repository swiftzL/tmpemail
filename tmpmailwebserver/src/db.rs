use sqlx::types::chrono::NaiveDateTime;
use sea_orm::*;
use std::time::Duration;
use once_cell::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::new();

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "temp_emails")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub email: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub content: Option<String>,
    pub subject: Option<String>,
    pub from_email:Option<String>,
    pub from_name:Option<String>
}

#[derive(Debug, FromQueryResult)]
pub struct EmailInfo {
    pub id: u32,
    pub email: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub subject: Option<String>,
    pub from_email:Option<String>,
    pub from_name:Option<String>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn find_by_email(email: &str) -> Result<Vec<EmailInfo>, DbErr> {
    let db = DB.get().expect("Database not initialized");
    Entity::find()
        .filter(Column::Email.eq(email))
        .order_by_desc(Column::CreatedAt)
        .into_model::<EmailInfo>()
        .all(db)
        .await
}

pub async fn find_by_id(id: u32) -> Result<Option<Model>, DbErr> {
    let db = DB.get().expect("Database not initialized");
    Entity::find_by_id(id).one(db).await
}

pub async fn init_db() -> Result<(), DbErr> {
    // let database_url = "mysql://root:123456@192.168.254.253:3306/tempemail";
    let database_url = "mysql://root:root123@1adcsfsa@127.0.0.1:3306/tempemail";

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(10)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false);

    let conn = Database::connect(opt).await?;
    DB.set(conn).expect("Failed to set database connection");
    Ok(())
}
