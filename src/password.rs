use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct Password<'a> {
    salt: SaltString,
    argon2: Argon2<'a>,
}

impl<'a> Password<'a> {
    pub fn new() -> Self {
        Self {
            salt: SaltString::generate(&mut OsRng),
            argon2: Argon2::default(),
        }
    }

    pub fn hash_password(&self, password: &[u8]) -> argon2::password_hash::Result<String> {
        Ok(self.argon2.hash_password(password, &self.salt)?.to_string())
    }

    pub fn verify_password(
        &self,
        password: &[u8],
        hash: PasswordHash,
    ) -> argon2::password_hash::Result<()> {
        self.argon2.verify_password(password, &hash)
    }
}
