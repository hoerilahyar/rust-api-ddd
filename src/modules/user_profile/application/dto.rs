use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::user_profile::domain::UserProfile;

/// `PUT /me/profile` body -- create-or-replace, so a user can fill in
/// their profile for the first time with the same request they'd use to
/// update it later. Every field is optional/independent; omitted fields
/// keep their current stored value (see `UserProfileRepository::upsert`).
#[derive(Debug, Deserialize, Validate)]
pub struct UpsertUserProfileRequest {
    #[validate(length(max = 30, message = "phone must be at most 30 characters"))]
    pub phone: Option<String>,

    #[validate(length(max = 500, message = "address must be at most 500 characters"))]
    pub address: Option<String>,

    #[validate(length(max = 100, message = "city must be at most 100 characters"))]
    pub city: Option<String>,

    #[validate(length(max = 100, message = "country must be at most 100 characters"))]
    pub country: Option<String>,

    #[validate(length(max = 20, message = "postal_code must be at most 20 characters"))]
    pub postal_code: Option<String>,

    #[validate(length(max = 20, message = "gender must be at most 20 characters"))]
    pub gender: Option<String>,

    pub date_of_birth: Option<NaiveDate>,

    #[validate(length(max = 255, message = "avatar_url must be at most 255 characters"))]
    pub avatar_url: Option<String>,

    #[validate(length(max = 255, message = "website must be at most 255 characters"))]
    pub website: Option<String>,

    #[validate(length(max = 1000, message = "bio must be at most 1000 characters"))]
    pub bio: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user_id: i32,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub gender: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserProfile> for UserProfileResponse {
    fn from(p: UserProfile) -> Self {
        Self {
            user_id: p.user_id,
            phone: p.phone,
            address: p.address,
            city: p.city,
            country: p.country,
            postal_code: p.postal_code,
            gender: p.gender,
            date_of_birth: p.date_of_birth,
            avatar_url: p.avatar_url,
            website: p.website,
            bio: p.bio,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
