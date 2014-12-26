/* vim: set et: */

#![feature(macro_rules)]

extern crate url;
extern crate serialize;

#[cfg(test)]
use serialize::json;
#[cfg(test)]
use std::io::{BufferedReader, File};
#[cfg(test)]
use types::recordinginfo::RecordingInfo;

mod types;

#[test]
fn able_to_parse_recordinginfo() -> () {
    let data = BufferedReader::new(File::open(&Path::new("testdata/recordinginfo.txt"))).read_line();
    match data {
        Ok(line) => {
            match json::decode(line.as_slice()) {
                Ok(ri) => {
                    let rinfo : RecordingInfo = ri;
                    assert!(rinfo.get_id() == 1000001);
                    println!("{}", rinfo.get_name());
                    assert!(rinfo.get_name() == "Tämä on testi");
                    assert!(rinfo.get_channel() == "MTV3");
                    assert!(rinfo.get_length() == 5);
                    // start_time
                    // end_time
                    assert!(rinfo.get_url().to_string() == "http://google.fi/");
                    assert!(rinfo.get_program_view_id() == 123456789);
                    assert!(rinfo.get_recording_id() == 987654321);
                },
                Err(err) => {
                    println!("{}", err);
                    assert!(false);
                }
            };
        },
        Err(err) => {
            println!("{}", err);
            assert!(false);
        }
    }
}
