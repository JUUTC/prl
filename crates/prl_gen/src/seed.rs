//! Versioned generation domains, separate from the gameplay RNG.

use prl_content::ContentId;
use rand_chacha::ChaCha8Rng;
use rand_core::SeedableRng;

pub(super) fn crypt_rng(seed: u64, map: &ContentId, floor: u32) -> ChaCha8Rng {
    ChaCha8Rng::from_seed(crypt_seed(seed, map, floor))
}

fn crypt_seed(seed: u64, map: &ContentId, floor: u32) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("PRL crypt layout seed v1");
    hasher.update(&seed.to_le_bytes());
    hasher.update(&floor.to_le_bytes());
    // Namespaced IDs cannot contain NUL; a terminator makes the encoding explicit.
    hasher.update(map.to_string().as_bytes());
    hasher.update(&[0]);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use rand_core::Rng;

    use super::*;

    #[test]
    fn crypt_domain_encoding_has_a_portable_golden() {
        let map = ContentId::new("prl", "playtest_crypt").unwrap();
        assert_eq!(
            blake3::Hash::from_bytes(crypt_seed(42, &map, 1))
                .to_hex()
                .as_str(),
            "d9a38c3faad0bb92436ff512dd1e857f2c181418d6848ddd7ac0be984930a484"
        );
    }

    #[test]
    fn seed_map_and_floor_separate_generation_streams() {
        let map = ContentId::new("prl", "crypt").unwrap();
        let other = ContentId::new("mod", "crypt").unwrap();
        let expected = crypt_seed(42, &map, 1);
        assert_ne!(expected, crypt_seed(43, &map, 1));
        assert_ne!(expected, crypt_seed(42, &map, 2));
        assert_ne!(expected, crypt_seed(42, &other, 1));
        let mut first = crypt_rng(42, &map, 1);
        let mut repeated = crypt_rng(42, &map, 1);
        assert!((0..100).all(|_| first.next_u32() == repeated.next_u32()));
    }
}
