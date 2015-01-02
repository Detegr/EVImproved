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
    recordings_count: uint
}

impl FolderInfo {
    fn root(rec_count: uint) -> FolderInfo {
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
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, Some(self.folder.folders.len()))
    }
}

impl<'a> Iterator<Recording> for RecordingIter<'a> {
    fn next(&mut self) -> Option<Recording> {
        assert!(self.index < self.folder.recordings.len());
        None
    }
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, Some(self.folder.recordings.len()))
    }
}

impl<E, D : Decoder<E>> Folder {
    fn decode_folder(d: &mut D) -> Result<Folder, E> {
        let recordings: Vec<RecordingInfo> = json_field!("recordings", d);
        Ok(Folder {
            info: FolderInfo::root(recordings.len()),
            folders: json_field!("folders", d),
            recordings: recordings
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
        println!("{}", f.folder_iter().size_hint());
        assert!(f.folder_iter().size_hint() == (0, Some(2)));
        for fldr in f.folder_iter() {
            match fldr.get_id() {
                10000001 | 10000002 => {},
                _ => assert!(false)
            }
        }
    });
}
