/**
 * password.rs
 * Password related functions.
*/
use rand::{Rng, SeedableRng};
use rand_hc::Hc128Rng;

pub struct Password {
    pub pass: String,
    pub len: usize,
}

impl Password {
    pub fn new(pass: String, len: usize) -> Password {
        Password {
            pass: String::from(pass),
            len: len,
        }
    }

    pub fn generate(mut pass_len: usize) -> Password {
        if pass_len < 8 {
            pass_len = 8;
        }
        if pass_len > 128 {
            pass_len = 128;
        }

        let ascii_chars: &str = "!#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKL1MNOPQRSTUVWXYZ[]^_abcdefghijklmnopqrstuvwxyz}{|~";
        let mut rng = Hc128Rng::from_entropy();

        let mut pass: String = String::new();
        for _ in 0..pass_len {
            let rnd_chr = rng.gen_range(0..ascii_chars.len());
            let ascii_char_byte: u8 = ascii_chars.as_bytes()[rnd_chr as usize];
            let character: char = ascii_char_byte as char;
            pass.push(character);
        }
        let generated_password = Password::new(pass, pass_len);
        generated_password
    }
}
