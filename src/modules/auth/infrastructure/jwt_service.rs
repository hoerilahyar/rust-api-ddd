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
        if data.claims.token_type != TokenType::Access {
            return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
        }
        Ok(data.claims)
    }

    pub fn decode_refresh_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(token, &self.refresh_decoding_key, &Validation::default())?;
        if data.claims.token_type != TokenType::Refresh {
            return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
        }
        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::config::JwtConfig;

    fn service() -> JwtService {
        JwtService::new(&JwtConfig {
            access_secret: "test-access-secret".to_string(),
            refresh_secret: "test-refresh-secret".to_string(),
            access_ttl_seconds: 900,
            refresh_ttl_seconds: 604_800,
        })
    }

    fn user() -> UserAuthProjection {
        UserAuthProjection {
            id: 42,
            name: "Jane Doe".to_string(),
            username: "jane".to_string(),
            email: "jane@example.com".to_string(),
            password_hash: "irrelevant-for-jwt".to_string(),
            is_active: true,
            roles: vec!["admin".to_string()],
            permissions: vec!["user.manage".to_string()],
        }
    }

    #[test]
    fn access_token_roundtrips_and_carries_claims() {
        let svc = service();
        let (token, ttl) = svc.generate_access_token(&user()).unwrap();
        assert_eq!(ttl, 900);

        let claims = svc.decode_access_token(&token).unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.username, "jane");
        assert_eq!(claims.roles, vec!["admin".to_string()]);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn refresh_token_roundtrips_and_carries_claims() {
        let svc = service();
        let (token, ttl) = svc.generate_refresh_token(&user()).unwrap();
        assert_eq!(ttl, 604_800);

        let claims = svc.decode_refresh_token(&token).unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    /// Regression test for the token_type validation fix: previously
    /// `decode_access_token`/`decode_refresh_token` only checked the
    /// signature and expiry, never `token_type`, so a refresh token could
    /// be used to pass `require_auth` if the two secrets were ever
    /// misconfigured to be equal. These two tests use the SAME secret for
    /// both sides to simulate that misconfiguration and confirm the type
    /// check alone is what blocks the swap.
    #[test]
    fn access_token_rejected_when_decoded_as_refresh_even_with_shared_secret() {
        let shared_secret = "only-one-secret".to_string();
        let svc = JwtService::new(&JwtConfig {
            access_secret: shared_secret.clone(),
            refresh_secret: shared_secret,
            access_ttl_seconds: 900,
            refresh_ttl_seconds: 604_800,
        });

        let (access_token, _) = svc.generate_access_token(&user()).unwrap();
        let result = svc.decode_refresh_token(&access_token);

        assert!(result.is_err(), "an access token must never pass as a refresh token");
    }

    #[test]
    fn refresh_token_rejected_when_decoded_as_access_even_with_shared_secret() {
        let shared_secret = "only-one-secret".to_string();
        let svc = JwtService::new(&JwtConfig {
            access_secret: shared_secret.clone(),
            refresh_secret: shared_secret,
            access_ttl_seconds: 900,
            refresh_ttl_seconds: 604_800,
        });

        let (refresh_token, _) = svc.generate_refresh_token(&user()).unwrap();
        let result = svc.decode_access_token(&refresh_token);

        assert!(result.is_err(), "a refresh token must never pass as an access token");
    }

    #[test]
    fn decode_fails_with_wrong_secret() {
        let svc = service();
        let (token, _) = svc.generate_access_token(&user()).unwrap();

        let other = JwtService::new(&JwtConfig {
            access_secret: "a-completely-different-secret".to_string(),
            refresh_secret: "another-different-secret".to_string(),
            access_ttl_seconds: 900,
            refresh_ttl_seconds: 604_800,
        });

        assert!(other.decode_access_token(&token).is_err());
    }
}
