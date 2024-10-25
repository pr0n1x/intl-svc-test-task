// based on https://tiemenwaterreus.com/posts/implementing-base64-in-rust/
// the own implementetion is needed because this project have to be runnable on rust playground

const UPPERCASEOFFSET: i8 = 65;
const LOWERCASEOFFSET: i8 = 71;
const DIGITOFFSET: i8 = -4;

pub trait Alphabet {
    const SIXTY_SECOND_SYMBOL: i8;
    const SIXTY_THIRD_SYMBOL: i8;

    fn get_char_for_index(index: u8) -> Option<char> {
        let index = index as i8;
        
        let ascii_index = match index {
            0..=25 => index + UPPERCASEOFFSET,  // A-Z
            26..=51 => index + LOWERCASEOFFSET, // a-z
            52..=61 => index + DIGITOFFSET,     // 0-9
            62 => Self::SIXTY_SECOND_SYMBOL,    // + or -
            63 => Self::SIXTY_THIRD_SYMBOL,     // / or _

            _ => return None,
        } as u8;

        Some(ascii_index as char)
    }

    #[allow(dead_code)]
    fn get_index_for_char(character: char) -> Option<u8> {
        let character = character as i8;
        let base64_index = match character {
            65..=90 => character - UPPERCASEOFFSET,  // A-Z
            97..=122 => character - LOWERCASEOFFSET, // a-z
            48..=57 => character - DIGITOFFSET,      // 0-9
            _ if character == Self::SIXTY_SECOND_SYMBOL => 62, // + or -
            _ if character == Self::SIXTY_THIRD_SYMBOL => 63,  // / or _
            _ => return None,
        } as u8;

        Some(base64_index)
    }

    fn get_padding_char() -> char {
        '='
    }
}

pub struct Std;
pub struct Url;

impl Alphabet for Std {
    const SIXTY_SECOND_SYMBOL: i8 = '+' as i8;
    const SIXTY_THIRD_SYMBOL: i8 = '/' as i8;
}

#[allow(dead_code)]
impl Std {
    pub fn encode(data: &[u8]) -> String {
        encode::<Self>(data)
    }
}

impl Alphabet for Url {
    const SIXTY_SECOND_SYMBOL: i8 = '-' as i8;
    const SIXTY_THIRD_SYMBOL: i8 = '_' as i8;
}

impl Url {
    pub fn encode(data: &[u8]) -> String {
        encode::<Self>(data)
    }
}

pub fn encode<A: Alphabet + ?Sized>(data: &[u8]) -> String {
    let encoded = data
        .chunks(3)
        .map(split)
        .flat_map(|chunk| encode_chunk::<A>(chunk));
    String::from_iter(encoded)
}

fn split(chunk: &[u8]) -> Vec<u8> {
    match chunk.len() {
        1 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4
        ],

        2 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2,
        ],

        3 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2 | &chunk[2] >> 6,
            &chunk[2] & 0b00111111
        ],

        _ => unreachable!()
    }
}

fn encode_chunk<A: Alphabet + ?Sized>(chunk: Vec<u8>) -> Vec<char> {
    let mut out = vec![A::get_padding_char(); 4];
    for i in 0..chunk.len() {
        if let Some(chr) = A::get_char_for_index(chunk[i]) {
            out[i] = chr;
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(Url::encode("fluffy pancakes".as_bytes()), "Zmx1ZmZ5IHBhbmNha2Vz");
        assert_eq!(Std::encode("fluffy pancakes".as_bytes()), "Zmx1ZmZ5IHBhbmNha2Vz");
        assert_eq!(Url::encode("eightsym".as_bytes()), "ZWlnaHRzeW0=");
        assert_eq!(Std::encode("eightsym".as_bytes()), "ZWlnaHRzeW0=");
    }
}
