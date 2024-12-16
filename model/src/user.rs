use super::{error::ModelError, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use util::{error::UtilError,store::{RWDB,RODB},AppState};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone)]
pub struct NewUser {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone)]
pub struct UpdateUser {
    pub last_login: Option<DateTime<Utc>>,
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl User {
    pub async fn create(new_user: NewUser, db: RWDB) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            INSERT INTO users (role,display_name,email)
            values ($1,$2,$3)
            RETURNING id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            "#,
            new_user.role as Role,
            new_user.display_name,
            new_user.email
            )
            .fetch_one(db.get_conn())
            .await
            .map_err(|e| ModelError::from(UtilError::from(e)))
    }

    pub async fn update(
        updated_user: UpdateUser,
        db: RWDB
    ) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            UPDATE users
            SET 
            last_login = COALESCE($1,last_login),
            display_name = COALESCE($2,display_name), 
            email = COALESCE($3,email)
            RETURNING id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            "#,
            updated_user.last_login,
            updated_user.display_name,
            updated_user.email
            )
            .fetch_one(db.get_conn())
            .await
            .map_err(|e| ModelError::from(UtilError::from(e)))
    }

    pub async fn get(id: Uuid, db: RODB) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            FROM users
            where id = $1
            "#,
            id
            )
            .fetch_one(db.get_conn())
            .await
            .map_err(|e| ModelError::from(UtilError::from(e)))
    }

    pub async fn get_paginated(limit: i64,offset: i64,db: RODB) -> Result<Vec<User>, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            FROM users
            FETCH NEXT $1 ROWS ONLY
            OFFSET $2
            "#,
            limit,
            offset
            )
            .fetch_all(db.get_conn())
            .await
            .map_err(|e| ModelError::from(UtilError::from(e)))
    }
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn create() {
    }

    #[tokio::test]
    async fn update() {
    }

    #[tokio::test]
    async fn get() {
    }

    #[tokio::test]
    async fn get_paginated() {
    }
}
