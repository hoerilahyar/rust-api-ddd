use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::user_profile::domain::{UserProfile, UserProfileRepository};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`UserProfileRepository`], targeting the
/// `user_profiles` table (see `databases/postgresql/migrations`). Every
/// query filters on `user_id` -- see the trait doc for why.
#[derive(Clone)]
pub struct UserProfileRepositoryPg {
    pool: PgPool,
}

impl UserProfileRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> UserProfile {
        UserProfile {
            id: row.get("id"),
            user_id: row.get("user_id"),
            phone: row.get("phone"),
            address: row.get("address"),
            city: row.get("city"),
            country: row.get("country"),
            postal_code: row.get("postal_code"),
            gender: row.get("gender"),
            date_of_birth: row.get("date_of_birth"),
            avatar_url: row.get("avatar_url"),
            website: row.get("website"),
            bio: row.get("bio"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl UserProfileRepository for UserProfileRepositoryPg {
    async fn find_by_user_id(&self, user_id: i32) -> Result<Option<UserProfile>, AppError> {
        let row = sqlx::query("SELECT * FROM user_profiles WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    async fn upsert(
        &self,
        user_id: i32,
        phone: Option<&str>,
        address: Option<&str>,
        city: Option<&str>,
        country: Option<&str>,
        postal_code: Option<&str>,
        gender: Option<&str>,
        date_of_birth: Option<chrono::NaiveDate>,
        avatar_url: Option<&str>,
        website: Option<&str>,
        bio: Option<&str>,
    ) -> Result<UserProfile, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_profiles (
                user_id, phone, address, city, country, postal_code,
                gender, date_of_birth, avatar_url, website, bio
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (user_id) DO UPDATE
                SET phone         = COALESCE(EXCLUDED.phone, user_profiles.phone),
                    address       = COALESCE(EXCLUDED.address, user_profiles.address),
                    city          = COALESCE(EXCLUDED.city, user_profiles.city),
                    country       = COALESCE(EXCLUDED.country, user_profiles.country),
                    postal_code   = COALESCE(EXCLUDED.postal_code, user_profiles.postal_code),
                    gender        = COALESCE(EXCLUDED.gender, user_profiles.gender),
                    date_of_birth = COALESCE(EXCLUDED.date_of_birth, user_profiles.date_of_birth),
                    avatar_url    = COALESCE(EXCLUDED.avatar_url, user_profiles.avatar_url),
                    website       = COALESCE(EXCLUDED.website, user_profiles.website),
                    bio           = COALESCE(EXCLUDED.bio, user_profiles.bio)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(phone)
        .bind(address)
        .bind(city)
        .bind(country)
        .bind(postal_code)
        .bind(gender)
        .bind(date_of_birth)
        .bind(avatar_url)
        .bind(website)
        .bind(bio)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, user_id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM user_profiles WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
