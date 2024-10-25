use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};


pub trait SlugGenerator {
    /// we have to generate 32 bits of hash or random + 16bit bump
    /// result 48 bits value fits in 8 symbols in base64 without padding ('=' equal signs)
    fn generate(&self, input: &str, bump: u16) -> Slug;
}

///
pub struct SimplestSlugGenerator;

impl SlugGenerator for SimplestSlugGenerator {
    fn generate(&self, _input: &str, bump: u16) -> Slug {
        SimplestSlugGenerator::generate(&self, bump)
    }
}
impl SimplestSlugGenerator {
    fn generate(&self, bump: u16) -> Slug {
        // pseudo-random without using 'rand' crate
        let rand_bytes: [u8; 4] = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            .to_be_bytes();
        let mut result_bytes: [u8; 6] = [0, 0, 0, 0, 0, 0];
        result_bytes[..4].clone_from_slice(&rand_bytes);
        result_bytes[4..6].clone_from_slice(&bump.to_be_bytes());
        Slug::from(base64::Url::encode(&result_bytes))
    }
}

#[cfg(test)]
mod test {
    use crate::gen::SimplestSlugGenerator;

    #[test]
    fn test_generated_slug_len() {
        assert_eq!(SimplestSlugGenerator.generate(128).len(), 8)
    }
}
