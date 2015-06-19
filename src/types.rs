/* vim: set et: */

use hyper::client::Client;
use hyper::header::Headers;
use std::default::Default;
use std::fmt;
use std::io::Read;
use std::ops::Deref;
use url::Url;
use url::percent_encoding::percent_decode;
use urls::EVUrl;

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
#[derive(Clone, Debug, RustcDecodable)]
pub struct FolderInfo {
    id: i32, // TODO: Use FolderId and manually implement Decodable
    pub name: String,
    pub size: String, // TODO: Floating point
    pub has_unwatched: String, // TODO: Boolean
    pub has_wildcards: String, // TODO: Boolean
    pub has_pin: String, // TODO: option<int>
    pub recordings_count: usize
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
            name: "Root".into(),
            size: "0".into(),
            has_unwatched: "false".into(),
            has_wildcards: "false".into(),
            has_pin: "false".into(),
            recordings_count: rec_count
        }
    }
}
/// Represents an item returned by Folders iterator
pub struct FolderItem<'a> {
    session_headers: &'a Headers,
    folder_info: &'a FolderInfo,
}
impl<'a> FolderItem<'a> {
    #[cfg(not(test))]
    pub fn fetch(&self) -> Option<Folder> {
        // TODO: Use FolderId enum
        let url = match self.folder_info.id {
            0 => EVUrl::Folder(FolderId::Root),
            id => EVUrl::Folder(FolderId::FolderId(id))
        };
        let mut client = Client::new();
        let res = client.get(url).headers(self.session_headers.clone()).send();
        res.ok().and_then(|mut res| {
            let mut ok = String::new();
            if let Err(_) = res.read_to_string(&mut ok) {
                return None
            }
            json::decode(&ok).and_then(|mut f: Folder| {
                f.info = self.folder_info.clone();
                f.set_headers(self.session_headers);
                Ok(f)
            }).ok()
        })
    }
}
impl<'a> Deref for FolderItem<'a> {
    type Target = FolderInfo;
    fn deref<'b>(&'b self) -> &'b Self::Target {
        self.folder_info
    }
}

pub struct RecordingItem<'a> {
    session_headers: &'a Headers,
    recording_info: &'a RecordingInfo,
}
impl<'a> RecordingItem<'a> {
    #[cfg(not(test))]
    fn fetch(&self) -> Option<Recording> {
        None // NYI
    }
}
impl<'a> Deref for RecordingItem<'a> {
    type Target = RecordingInfo;
    fn deref<'b>(&'b self) -> &'b Self::Target {
        self.recording_info
    }
}


/// Folder in Elisa Viihde
#[allow(dead_code)]
#[derive(Debug)]
pub struct Folder {
    info: FolderInfo,
    folders: Vec<FolderInfo>,
    recordings: Vec<RecordingInfo>,
    session_headers: Headers
}

/// Iterator over folders in another folder
pub struct Folders<'a> {
    index: usize,
    folder: &'a Folder
}

/// Iterator over recordings in a folder
pub struct Recordings<'a> {
    index: usize,
    folder: &'a Folder
}

impl<'a> Folder {
    /// Returns Folders over this folder
    pub fn folders(&'a self) -> Folders<'a> {
        Folders { index: 0, folder: &self }
    }
    /// Returns Recordings over this folder
    pub fn recordings(&'a self) -> Recordings<'a> {
        Recordings { index: 0, folder: self }
    }
    /// TODO: Is this necessary to be public? (Sets http headers for subsequent calls using this folder)
    pub fn set_headers(&mut self, headers: &Headers) {
        self.session_headers = headers.clone();
    }
}

impl fmt::Display for Folder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.info.name)
    }
}

impl<'a> Iterator for Folders<'a> {
    type Item = FolderItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let items = self.folder.folders.len();
        if items!=0 && self.index < items-1 {
            self.index += 1;
            Some(FolderItem {
                session_headers: &self.folder.session_headers,
                folder_info: &self.folder.folders[self.index]
            })
        }
        else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.folder.folders.len()))
    }
}

impl<'a> Iterator for Recordings<'a> {
    type Item = RecordingItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let items = self.folder.recordings.len();
        if items!=0 && self.index < items-1 {
            self.index += 1;
            Some(RecordingItem {
                session_headers: &self.folder.session_headers,
                recording_info: &self.folder.recordings[self.index]
            })
        }
        else {
            return None
        }
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
            recordings: recordings,
            session_headers: Headers::new(),
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
#[derive(Debug)]
pub struct Recording {
    info: RecordingInfo,
    id: i32,
    name: String,
    channel: String,
    length: i32,
    start_time: String,
    end_time: String,
    url: Url,
    programviewid: i32,
    recordingid: i32
}

/// Contains information of a Recording
#[derive(Clone, Debug, RustcDecodable)]
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

impl Recording {
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

    impl<'a> super::FolderItem<'a> {
        #[cfg(test)]
        fn fetch(&self) -> Option<Folder> {
            use std::io::BufReader;
            let file = File::open(format!("testdata/folder_{}.json", self.folder_info.id)).unwrap();
            let line = BufReader::new(file).lines().next().unwrap().unwrap();
            let mut fldr: Folder = json::decode(&line).unwrap();
            fldr.info = self.folder_info.clone();
            Some(fldr)
        }
    }
    impl<'a> super::RecordingItem<'a> {
        #[cfg(test)]
        fn fetch(&self) -> Option<Recording> {
            use std::io::BufReader;
            let file = File::open(format!("testdata/recording_{}.json", self.recording_info.program_id)).unwrap();
            let line = BufReader::new(file).lines().next().unwrap().unwrap();
            let mut rec: Recording = json::decode(&line).unwrap();
            rec.info = self.recording_info.clone();
            Some(rec)
        }
    }


    #[test]
    fn able_to_parse_folder() {
        setup_test!("testdata/root_folder.json", |_ : Folder| {});
    }

    #[test]
    fn able_to_iterate_folders() {
        setup_test!("testdata/root_folder.json", |f : Folder| {
            assert!(f.folders().size_hint() == (0, Some(2)), "Folder iterator had invalid bounds");
            let mut has_items: bool = false;
            for fldr in f.folders() {
                has_items = true;
                match fldr.id {
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
            //assert!(f.recordings().size_hint() == (0, Some(2)));
            let mut has_items = false;
            for rec in f.recordings() {
                has_items = true;
                match rec.program_id {
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
            assert!(r.id == 1000001);
            assert!(r.name == "Tämä on testi"); // Finnish characters used on purpose
            assert!(r.channel == "MTV3");
            assert!(r.length == 5);
            // start_time
            // end_time
            assert!(r.url.to_string() == "http://google.fi/");
            assert!(r.programviewid == 123456789);
            assert!(r.recordingid == 987654321);
        });
    }
}

