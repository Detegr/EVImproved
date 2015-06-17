/* vim: set et: */

use std::default::Default;
use std::fmt;
use url::Url;
use url::percent_encoding::percent_decode;
use std::fs::File;
use std::path::PathBuf;
use std::io::BufRead;

#[allow(unused_imports)]
use rustc_serialize::{json,Decodable,Decoder};

macro_rules! json_field {
    ($name:expr, $decoder:expr) => {
        try!($decoder.read_struct_field($name, 0, |d| Decodable::decode(d)))
    }
}

/// Describes an id of an folder
pub enum FolderId {
    Root,
    FolderId(i32)
}
impl fmt::Display for FolderId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FolderId::Root => write!(fmt, "0"),
            FolderId::FolderId(ref id) => write!(fmt, "{}", id)
        }
    }
}

/// Contains information of a folder
#[allow(dead_code)]
#[derive(Clone, RustcDecodable)]
pub struct FolderInfo {
    id: i32,
    name: String,
    size: String, // TODO: Floating point
    has_unwatched: String, // TODO: Boolean
    has_wildcards: String, // TODO: Boolean
    has_pin: String, // TODO: option<int>
    recordings_count: usize
}
/*
impl<E, D : Decoder<E>> Decodable<D, E> for FolderInfo {
    fn decode(d: &mut D) -> Result<Folder, E> {
        let size_str = json_value!("size", d).words().next();
        let size = Float::parse(size_str.as_ref());
        Ok(FolderInfo {
            id: json_value!("id", d),
            name: json_value!("name", d),
            size: size
        })
    }
}*/

impl FolderInfo {
    fn root(rec_count: usize) -> FolderInfo {
        FolderInfo {
            id: 0,
            name: "Root".to_string(),
            size: "0".to_string(),
            has_unwatched: "false".to_string(),
            has_wildcards: "false".to_string(),
            has_pin: "false".to_string(),
            recordings_count: rec_count
        }
    }
}

/// Folder in Elisa Viihde
#[allow(dead_code)]
pub struct Folder {
    info: FolderInfo,
    folders: Vec<FolderInfo>,
    recordings: Vec<RecordingInfo>
}

/// Iterator over folders in another folder
pub struct FolderIter<'a> {
    index: usize,
    folder: &'a Folder
}

/// Iterator over recordings in a folder
pub struct RecordingIter<'a> {
    index: usize,
    folder: &'a Folder
}

impl<'a> Folder {
    fn get_id(&self) -> i32 {
        self.info.id
    }
    /// Returns FolderIter over this folder
    pub fn folder_iter(&'a self) -> FolderIter<'a> {
        FolderIter { index: 0, folder: self }
    }
    /// Returns RecordingIter over this folder
    pub fn rec_iter(&'a self) -> RecordingIter<'a> {
        RecordingIter { index: 0, folder: self }
    }
    #[cfg(not(test))]
    fn fetch_folder(&self, fi: &FolderInfo) -> Option<Folder> {
        None // NYI
    }
    #[cfg(not(test))]
    fn fetch_recording(&self, ri: &RecordingInfo) -> Option<Recording> {
        None // NYI
    }
    #[cfg(test)]
    fn fetch_folder(&self, fi: &FolderInfo) -> Option<Folder> {
        use std::io::BufReader;
        let file = File::open(format!("testdata/folder_{}.json", fi.id)).unwrap();
        let line = BufReader::new(file).lines().next().unwrap().unwrap();
        let mut fldr: Folder = json::decode(&line).unwrap();
        fldr.info = fi.clone();
        Some(fldr)
    }
    #[cfg(test)]
    fn fetch_recording(&self, ri: &RecordingInfo) -> Option<Recording> {
        use std::io::BufReader;
        let file = File::open(format!("testdata/recording_{}.json", ri.program_id)).unwrap();
        let line = BufReader::new(file).lines().next().unwrap().unwrap();
        let mut rec: Recording = json::decode(&line).unwrap();
        rec.info = ri.clone();
        Some(rec)
    }
}

impl<'a> Iterator for FolderIter<'a> {
    #![cfg(test)]
    type Item = Folder;
    fn next(&mut self) -> Option<Folder> {
        if self.index >= self.folder.folders.len() {
            return None
        }
        let fi = &self.folder.folders[self.index];
        self.index += 1;
        self.folder.fetch_folder(fi)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.folder.folders.len()))
    }
}

impl<'a> Iterator for RecordingIter<'a> {
    type Item = Recording;
    fn next(&mut self) -> Option<Recording> {
        if self.index >= self.folder.recordings.len() {
            return None
        }
        let ri = &self.folder.recordings[self.index];
        self.index += 1;
        self.folder.fetch_recording(ri)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.folder.recordings.len()))
    }
}

impl Folder {
    fn decode_folder<D : Decoder>(d: &mut D) -> Result<Folder, D::Error> {
        let recordings: Vec<RecordingInfo> = json_field!("recordings", d);
        Ok(Folder {
            info: FolderInfo::root(recordings.len()),
            folders: json_field!("folders", d),
            recordings: recordings
        })
    }
}

impl Decodable for Folder {
    fn decode<D : Decoder>(d: &mut D) -> Result<Folder, D::Error> {
        d.read_struct("", 0, |d| {
            d.read_struct_field("ready_data", 0, |rd| {
                rd.read_seq(|rd, len| {
                    assert!(len==1);
                    rd.read_seq_elt(0, |rd| { Folder::decode_folder(rd) })
                })
            })
        })
    }
}

/// Id of a program in Elisa Viihde
pub enum ProgramId {
    ProgramId(i32)
}
impl fmt::Display for ProgramId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProgramId::ProgramId(ref id) => write!(fmt, "{}", id)
        }
    }
}

/// Recording in Elisa Viihde
#[allow(dead_code)]
pub struct Recording {
    info: RecordingInfo,
    id : i32,
    name : String,
    channel : String,
    length : i32,
    start_time : String,
    end_time : String,
    url : Url,
    programviewid : i32,
    recordingid : i32
}

/// Contains information of a Recording
#[derive(Clone, RustcDecodable)]
#[allow(dead_code)]
pub struct RecordingInfo {
    id : i32,
    program_id : i32,
    folder_id : String, // TODO: Option<int>
    name: String,
    channel: String,
    start_time: String, // TODO
    timestamp: String, // TODO
    viewcount: i32,
    length: i32
}

impl Default for RecordingInfo {
    fn default() -> RecordingInfo {
        RecordingInfo {
            id: 0,
            program_id: 0,
            folder_id: "".to_string(),
            name: "".to_string(),
            channel: "".to_string(),
            start_time: "".to_string(),
            timestamp: "".to_string(),
            viewcount: 0,
            length: 0
        }
    }
}

impl Decodable for Recording {
    fn decode<D : Decoder>(d: &mut D) -> Result<Recording, D::Error> {
        d.read_struct("", 0, |d| {
            Ok(Recording {
                info: Default::default(),
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
    pub fn get_id(&self) -> i32 { self.id }
    pub fn get_name(&self) -> &str { self.name.as_ref() }
    pub fn get_channel(&self) -> &str { self.channel.as_ref() }
    pub fn get_length(&self) -> i32 { self.length }
    pub fn get_start_time(&self) -> &str { self.start_time.as_ref() } // TODO
    pub fn get_end_time(&self) -> &str { self.end_time.as_ref() } // TODO
    pub fn get_url(&self) -> &Url { &self.url }
    pub fn get_program_view_id(&self) -> i32 { self.programviewid }
    pub fn get_recording_id(&self) -> i32 { self.recordingid }
}

#[cfg(test)]
mod tests {
    use rustc_serialize::json;
    use std::io::BufReader;
    use super::{Recording, Folder};
    use std::fs::File;
    use std::io::Lines;
    use std::io::BufRead;

    macro_rules! setup_test(
        ($filename:expr, $code:expr) => {
            match BufReader::new(File::open($filename).unwrap()).lines().next().unwrap() {
                Ok(line) => {
                    println!("{}", line);
                    match json::decode(&line) {
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
    fn able_to_parse_folder() {
        setup_test!("testdata/root_folder.json", |_ : Folder| {});
    }

    #[test]
    fn able_to_iterate_folders() {
        setup_test!("testdata/root_folder.json", |f : Folder| {
            assert!(f.folder_iter().size_hint() == (0, Some(2)), "Folder iterator had invalid bounds");
            let mut has_items: bool = false;
            for fldr in f.folder_iter() {
                has_items = true;
                match fldr.get_id() {
                    1000001 | 1000002 => {},
                    _ => assert!(false, "Folder id was invalid")
                }
            }
            assert!(has_items, "Folder iterator should return some items but it returned none");
        });
    }

    #[test]
    fn able_to_iterate_recordings() {
        setup_test!("testdata/root_folder.json", |f : Folder| {
            //assert!(f.rec_iter().size_hint() == (0, Some(2)));
            let mut has_items = false;
            for rec in f.rec_iter() {
                has_items = true;
                match rec.info.program_id {
                    1000001 | 1000002 => {},
                    _ => assert!(false, "Program id was invalid")
                }
            }
            assert!(has_items, "Recording iterator should return some items but it returned none");
        });
    }

    #[test]
    fn able_to_parse_recording() -> () {
        setup_test!("testdata/recording_1000001.json", |r : Recording| {
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
}

