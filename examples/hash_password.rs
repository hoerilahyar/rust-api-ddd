//! Small dev utility: prints an Argon2 hash for a password so you can seed
//! or reset the admin user without spinning up the whole server.
//!
//! Usage:
//! cargo run --example hash_password -- "YourP@ssw0rd"
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
fn main() {
    let password = std::env::args()
        .nth(1)
        .expect("usage: cargo run --example hash_password -- <password>");
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string();
    println!("{hash}");
}
