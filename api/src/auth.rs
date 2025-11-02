use actix_web::{web, HttpResponse, Responder, HttpRequest, dev::ServiceRequest, Error};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

pub static USERS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

fn hash_password(password: &str) -> String {
    use argon2::Argon2;
    use password_hash::SaltString;
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(hash: &str, password: &str) -> bool {
    use argon2::Argon2;
    use password_hash::PasswordHash;

    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &parsed).is_ok()
}

fn jwt_secret() -> &'static [u8] {
    // Prefer environment variable `EVENTHUB_JWT_SECRET`, fallback to a default for tests.
    use once_cell::sync::Lazy;
    static SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
        std::env::var("EVENTHUB_JWT_SECRET").map(|s| s.into_bytes()).unwrap_or_else(|_| b"eventhub-secret-please-change".to_vec())
    });
    &SECRET
}

fn make_jwt(email: &str) -> String {
    use jsonwebtoken::{EncodingKey, Header};
    use time::OffsetDateTime;

    #[derive(Serialize)]
    struct Claims<'a> {
        sub: &'a str,
        exp: i64,
    }

    let exp = (OffsetDateTime::now_utc() + time::Duration::minutes(60)).unix_timestamp();
    let claims = Claims { sub: email, exp };
    jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret())).unwrap()
}

pub async fn signup(body: web::Json<AuthRequest>) -> impl Responder {
    let mut users = USERS.lock().unwrap();
    if users.contains_key(&body.email) {
        return HttpResponse::Conflict().json(serde_json::json!({"error":"user exists"}));
    }
    let hash = hash_password(&body.password);
    users.insert(body.email.clone(), hash);
    HttpResponse::Created().finish()
}

pub async fn login(body: web::Json<AuthRequest>) -> impl Responder {
    let users = USERS.lock().unwrap();
    match users.get(&body.email) {
        Some(hash) if verify_password(hash, &body.password) => {
            let token = make_jwt(&body.email);
            HttpResponse::Ok().json(TokenResponse { token })
        }
        _ => HttpResponse::Unauthorized().json(serde_json::json!({"error":"invalid credentials"})),
    }
}

pub fn auth_middleware(req: ServiceRequest) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    use jsonwebtoken::{DecodingKey, Validation};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Claims { sub: String, exp: i64 }

    let headers = req.headers();
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret()), &Validation::default()) {
                    Ok(_) => return Ok(req),
                    Err(_) => {
                        let err = actix_web::error::ErrorUnauthorized("invalid token");
                        return Err((err, req));
                    }
                }
            }
        }
    }
    let err = actix_web::error::ErrorUnauthorized("missing token");
    Err((err, req))
}

use actix_web::http::header::AUTHORIZATION;
use password_hash::{PasswordHasher, PasswordVerifier};

/// Validate Authorization header (Bearer <token>) and return HttpResponse::Unauthorized on failure.
pub fn authorize_request(req: &HttpRequest) -> Result<(), HttpResponse> {
    use jsonwebtoken::{DecodingKey, Validation};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Claims { sub: String, exp: i64 }

    if let Some(hv) = req.headers().get(AUTHORIZATION) {
        if let Ok(s) = hv.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                return match jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret()), &Validation::default()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(HttpResponse::Unauthorized().json(serde_json::json!({"error":"invalid token"}))),
                };
            }
        }
    }
    Err(HttpResponse::Unauthorized().json(serde_json::json!({"error":"missing token"})))
}
