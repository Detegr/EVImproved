/* vim: set et: */

#![feature(macro_rules)]

extern crate url;
extern crate "rustc-serialize" as rustc_serialize;

#[cfg(test)]
use rustc_serialize::json;
#[cfg(test)]
use std::io::{BufferedReader, File};
#[cfg(test)]
use types::folder::Folder;
#[cfg(test)]
use types::recording::Recording;

mod types;

#[test]
macro_rules! setup_test(
    ($filename:expr, $code:expr) => {
        match BufferedReader::new(File::open(&Path::new($filename))).read_line() {
            Ok(line) => {
                println!("{}", line.as_slice());
                match json::decode(line.as_slice()) {
                    Ok(res) => {
                        $code(res)
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
);

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

#[test]
fn able_to_parse_readydata() -> () {
    setup_test!("testdata/readydata.json", |_ : Folder| {
    });
}
