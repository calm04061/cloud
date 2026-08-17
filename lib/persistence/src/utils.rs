use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hashed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "admin";
        let hashed = hash_password(password).unwrap();
        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("wrong", &hashed).unwrap());
    }
}
