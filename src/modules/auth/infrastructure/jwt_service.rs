use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::bootstrap::config::JwtConfig;
use crate::modules::auth::domain::value_object::{Claims, TokenType};
use crate::shared::contracts::UserAuthProjection;

/// Encodes/decodes JWT access & refresh tokens. Access and refresh tokens
/// use different secrets so a leaked refresh secret can't be used to mint
/// access tokens and vice versa.
#[derive(Clone)]
pub struct JwtService {
    access_encoding_key: EncodingKey,
    access_decoding_key: DecodingKey,
    refresh_encoding_key: EncodingKey,
    refresh_decoding_key: DecodingKey,
    access_ttl_seconds: i64,
    refresh_ttl_seconds: i64,
}

impl JwtService {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            access_encoding_key: EncodingKey::from_secret(config.access_secret.as_bytes()),
            access_decoding_key: DecodingKey::from_secret(config.access_secret.as_bytes()),
            refresh_encoding_key: EncodingKey::from_secret(config.refresh_secret.as_bytes()),
            refresh_decoding_key: DecodingKey::from_secret(config.refresh_secret.as_bytes()),
            access_ttl_seconds: config.access_ttl_seconds,
            refresh_ttl_seconds: config.refresh_ttl_seconds,
        }
    }

    fn build_claims(&self, user: &UserAuthProjection, token_type: TokenType, ttl_seconds: i64) -> Claims {
        let now = Utc::now().timestamp();
        Claims {
            sub: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
            token_type,
            iat: now,
            exp: now + ttl_seconds,
        }
    }

    pub fn generate_access_token(&self, user: &UserAuthProjection) -> Result<(String, i64), jsonwebtoken::errors::Error> {
        let claims = self.build_claims(user, TokenType::Access, self.access_ttl_seconds);
        let token = encode(&Header::default(), &claims, &self.access_encoding_key)?;
        Ok((token, self.access_ttl_seconds))
    }

    pub fn generate_refresh_token(&self, user: &UserAuthProjection) -> Result<(String, i64), jsonwebtoken::errors::Error> {
        let claims = self.build_claims(user, TokenType::Refresh, self.refresh_ttl_seconds);
        let token = encode(&Header::default(), &claims, &self.refresh_encoding_key)?;
        Ok((token, self.refresh_ttl_seconds))
    }

    pub fn decode_access_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(token, &self.access_decoding_key, &Validation::default())?;
        Ok(data.claims)
    }

    pub fn decode_refresh_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(token, &self.refresh_decoding_key, &Validation::default())?;
        Ok(data.claims)
    }
}
