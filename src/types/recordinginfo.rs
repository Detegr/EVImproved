/* vim: set et: */

use url::Url;
use url::percent_encoding::percent_decode;
use serialize::{Decodable,Decoder};

#[allow(dead_code)]
pub struct RecordingInfo {
    id : int,
    name : String,
    channel : String,
    length : int,
    start_time : String,
    end_time : String,
    url : Url,
    programviewid : int,
    recordingid : int
}

macro_rules! json_field {
    ($name:expr, $decoder:expr) => {
        try!($decoder.read_struct_field($name, 0, |d| Decodable::decode(d)))
    }
}

impl<E, D : Decoder<E>> Decodable<D, E> for RecordingInfo {
    fn decode(d: &mut D) -> Result<RecordingInfo, E> {
        d.read_struct("", 0, |d| {
            Ok(RecordingInfo {
                id: json_field!("id", d),
                name: {
                    let percent_encoded_str : String = json_field!("name", d);
                    String::from_utf8(percent_decode(percent_encoded_str.as_bytes())).unwrap()
                },
                channel: json_field!("channel", d),
                length: json_field!("length", d),
                start_time: json_field!("start_time", d),
                end_time: json_field!("end_time", d),
                url: json_field!("url", d),
                programviewid: json_field!("programviewid", d),
                recordingid: json_field!("recordingid", d)
            })
        })
    }
}

#[allow(dead_code)]
impl RecordingInfo {
    pub fn get_id(&self) -> int { self.id }
    pub fn get_name(&self) -> &str { self.name.as_slice() }
    pub fn get_channel(&self) -> &str { self.channel.as_slice() }
    pub fn get_length(&self) -> int { self.length }
    pub fn get_start_time(&self) -> &str { self.start_time.as_slice() } // TODO
    pub fn get_end_time(&self) -> &str { self.end_time.as_slice() } // TODO
    pub fn get_url(&self) -> &Url { &self.url }
    pub fn get_program_view_id(&self) -> int { self.programviewid }
    pub fn get_recording_id(&self) -> int { self.recordingid }
}
