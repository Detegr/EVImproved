use serialize::{Decodable,Decoder};
use url::percent_encoding::percent_decode;

pub struct PercentEncodedString {
    payload : String
}
impl PercentEncodedString {
    pub fn new(s : String) -> PercentEncodedString {
        PercentEncodedString {
            payload: String::from_utf8(percent_decode(s.as_bytes())).unwrap()
        }
    }
}
impl<E, D : Decoder<E>> Decodable<D, E> for PercentEncodedString {
    fn decode(d: &mut D) -> Result<PercentEncodedString, E> {
        d.read_str().map(|s| PercentEncodedString::new(s))
    }
}
impl Str for PercentEncodedString {
    fn as_slice(&self) -> &str {
        self.payload.as_slice()
    }
}
