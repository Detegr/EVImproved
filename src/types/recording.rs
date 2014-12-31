/* vim: set et: */

use std::fmt;
use url::Url;
use url::percent_encoding::percent_decode;

#[allow(unused_imports)]
use rustc_serialize::{json,Decodable,Decoder};

#[cfg(test)]
use std::io::{BufferedReader, File};

pub enum ProgramId {
    ProgramId(int)
}
impl fmt::Show for ProgramId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProgramId::ProgramId(ref id) => write!(fmt, "{}", id)
        }
    }
}

#[allow(dead_code)]
pub struct Recording {
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

#[deriving(RustcDecodable)]
#[allow(dead_code)]
pub struct RecordingInfo {
    id : int,
    program_id : int,
    folder_id : String, // TODO: Option<int>
    name: String,
    channel: String,
    start_time: String, // TODO
    timestamp: String, // TODO
    viewcount: int,
    length: int
}

impl<E, D : Decoder<E>> Decodable<D, E> for Recording {
    fn decode(d: &mut D) -> Result<Recording, E> {
        d.read_struct("", 0, |d| {
            Ok(Recording {
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
impl Recording {
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

#[test]
fn able_to_parse_recording() -> () {
    setup_test!("testdata/recording.json", |r : Recording| {
        assert!(r.get_id() == 1000001);
        assert!(r.get_name() == "Tämä on testi"); // Finnish characters used on purpose
        assert!(r.get_channel() == "MTV3");
        assert!(r.get_length() == 5);
        // start_time
        // end_time
        assert!(r.get_url().to_string() == "http://google.fi/");
        assert!(r.get_program_view_id() == 123456789);
        assert!(r.get_recording_id() == 987654321);
    });
}
