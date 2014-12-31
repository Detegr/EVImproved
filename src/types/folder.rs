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
    folders: Vec<FolderInfo>,
    recordings: Vec<RecordingInfo>
}

impl<E, D : Decoder<E>> Folder {
    fn decode_folder(d: &mut D) -> Result<Folder, E> {
        Ok(Folder {
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
    setup_test!("testdata/readydata.json", |_ : Folder| {
    });
}
