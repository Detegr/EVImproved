/* vim: set et: */

use std::fmt;
use super::recording::{Recording,RecordingInfo};

#[allow(unused_imports)]
use rustc_serialize::{json,Decodable,Decoder};

#[cfg(test)]
use std::io::{BufferedReader, File};

pub enum FolderId {
    Root,
    FolderId(int)
}
impl fmt::Show for FolderId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FolderId::Root => write!(fmt, "0"),
            FolderId::FolderId(ref id) => write!(fmt, "{}", id)
        }
    }
}

#[allow(dead_code)]
#[deriving(RustcDecodable)]
pub struct FolderInfo {
    id: int,
    name: String,
    size: String, // TODO: Floating point
    has_unwatched: String, // TODO: Boolean
    has_wildcards: String, // TODO: Boolean
    has_pin: String, // TODO: option<int>
    recordings_count: int
}

#[allow(dead_code)]
pub struct Folder {
    info: FolderInfo,
    folders: Vec<FolderInfo>,
    recordings: Vec<RecordingInfo>
}

pub struct FolderIter<'a> {
    index: uint,
    folder: &'a Folder
}
pub struct RecordingIter<'a> {
    index: uint,
    folder: &'a Folder
}

impl<'a> Folder {
    fn get_id(&self) -> int {
        self.info.id
    }
    fn folder_iter(&'a self) -> FolderIter<'a> {
        FolderIter { index: 0, folder: self }
    }
    fn rec_iter(&'a self) -> RecordingIter<'a> {
        RecordingIter { index: 0, folder: self }
    }
}

impl<'a> Iterator<Folder> for FolderIter<'a> {
    fn next(&mut self) -> Option<Folder> {
        assert!(self.index < self.folder.folders.len());
        let fi = &self.folder.folders[self.index];
        self.index += 1;
        // TODO: Fetch folder using fi
        None
    }
}
impl<'a> Iterator<Recording> for RecordingIter<'a> {
    fn next(&mut self) -> Option<Recording> {
        assert!(self.index < self.folder.folders.len() + self.folder.recordings.len());
        None
    }
}

impl<E, D : Decoder<E>> Folder {
    fn decode_folder(d: &mut D) -> Result<Folder, E> {
        Ok(Folder {
            info: FolderInfo { // TODO
                id: 0,
                name: "".to_string(),
                size: "".to_string(),
                has_unwatched: "".to_string(),
                has_wildcards: "".to_string(),
                has_pin: "".to_string(),
                recordings_count: 0
            },
            folders: json_field!("folders", d),
            recordings: json_field!("recordings", d),
        })
    }
}

impl<E, D : Decoder<E>> Decodable<D, E> for Folder {
    fn decode(d: &mut D) -> Result<Folder, E> {
        d.read_struct("", 0, |d| {
            // Try decoding ready_data first, if not found, decode normal folder
            d.read_struct_field("ready_data", 0, |rd| {
                rd.read_seq(|rd, len| {
                    assert!(len==1);
                    rd.read_seq_elt(0, |rd| { Folder::decode_folder(rd) })
                })
            }).or(Folder::decode_folder(d))
        })
    }
}

#[test]
fn able_to_parse_readydata() -> () {
    setup_test!("testdata/readydata.json", |_ : Folder| {});
}

#[test]
fn able_to_iterate_folders() -> () {
    setup_test!("testdata/readydata.json", |f : Folder| {
        assert!(f.folder_iter().size_hint() == (0, Some(1)));
        for fldr in f.folder_iter() {
            assert!(fldr.get_id() == 10000001);
        }
    });
}
