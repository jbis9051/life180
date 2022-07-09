use crate::DbPool;

use sqlx::postgres::{PgRow};
use sqlx::Row;

use rand_core::{OsRng, RngCore};

pub struct User {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub password: String,
    pub profile_picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub name: String,
    pub created: String,
}

impl User {
    pub async fn create(db: &DbPool, user: &User) -> Result<String, sqlx::Error> {
        sqlx::query(
            "INSERT INTO user (uuid, username, password, profile_picture, email, phone, name)
                             VALUES ($1, $2, $3, $4, $5, $6, $7);",
        )
        .bind(&user.uuid)
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.profile_picture)
        .bind(Option::<String>::None)
        .bind(&user.phone)
        .bind(&user.name)
        .execute(db)
        .await?;

        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let link_id = String::from_utf8_lossy(key.as_slice()).to_string();

        sqlx::query(
            "INSERT INTO confirmation (user_id, link_id, email)
                             VALUES ($1, $2, $3);",
        )
        .bind(&user.id)
        .bind(&link_id)
        .bind(&user.email)
        .execute(db)
        .await?;

        Ok(link_id)
    }

    pub async fn get_by_link_id(db: &DbPool, link_id: &str) -> Result<User, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM confirmation WHERE link_id IS $1;")
            .bind(link_id)
            .fetch_one(db)
            .await?;

        let user = User::user_by_row(&row).await;
        Ok(user)
    }

    pub async fn get_by_id(db: &DbPool, id: i32) -> Result<User, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM user WHERE id IS $1;")
            .bind(id)
            .fetch_one(db)
            .await?;

        let user = User::user_by_row(&row).await;
        Ok(user)
    }

    pub async fn get_by_uuid(db: &DbPool, uuid: &str) -> Result<User, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM user WHERE uuid IS $1;")
            .bind(uuid)
            .fetch_one(db)
            .await?;

        let user = User::user_by_row(&row).await;
        Ok(user)
    }

    /*
    fn update(&self, _conn: PoolConnection<Postgres>) {
        todo!();
    }
    fn delete(&self, _conn: PoolConnection<Postgres>) {
        todo!();
        // remove routes from a whole bunch of things
        // delete routes row
    }
    */
    async fn user_by_row(row: &PgRow) -> User {
        User {
            id: row.get("id"),
            uuid: row.get("uuid"),
            username: row.get("username"),
            password: row.get("password"),
            profile_picture: row.get("profile_picture"),
            email: row.get("email"),
            phone: row.get("phone"),
            name: row.get("name"),
            created: row.get("created"),
        }
    }
}
