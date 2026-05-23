/// One-shot helper to generate an Argon2 password hash.
///
/// Usage:
///   cargo run --bin gen_password_hash -- "ChangeMeASAP123!"
///
/// Copy the printed hash into the admin seed migration.
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let password = std::env::args()
        .nth(1)
        .expect("usage: cargo run --bin gen_password_hash -- <password>");

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hash failed")
        .to_string();

    println!("{hash}");
}
