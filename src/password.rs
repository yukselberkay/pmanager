/**
 * password.rs
 * Password related functions.
*/
use rand::rngs::OsRng;
use rand::Rng;
use zeroize::Zeroizing;

pub const MIN_GENERATED_LEN: usize = 8;
pub const MAX_GENERATED_LEN: usize = 128;

const ASCII_CHARS: &[u8] =
    b"!#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_abcdefghijklmnopqrstuvwxyz}{|~";

/// Generates a random password using the operating system's CSPRNG.
pub fn generate(pass_len: usize) -> Zeroizing<String> {
    let pass_len = pass_len.clamp(MIN_GENERATED_LEN, MAX_GENERATED_LEN);

    let mut pass = Zeroizing::new(String::with_capacity(pass_len));
    for _ in 0..pass_len {
        let rnd_chr = OsRng.gen_range(0..ASCII_CHARS.len());
        pass.push(ASCII_CHARS[rnd_chr] as char);
    }
    pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn charset_has_no_duplicates() {
        let unique: HashSet<_> = ASCII_CHARS.iter().collect();
        assert_eq!(unique.len(), ASCII_CHARS.len());
    }

    #[test]
    fn length_is_clamped() {
        assert_eq!(generate(1).len(), MIN_GENERATED_LEN);
        assert_eq!(generate(32).len(), 32);
        assert_eq!(generate(10_000).len(), MAX_GENERATED_LEN);
    }
}
