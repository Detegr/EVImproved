/* vim: set et: */

use headers::evimproved_headers;
use traits::Fetch;

#[cfg(not(test))]
use hyper::client::Client;
use hyper::header::Headers;
use std::default::Default;
use std::fmt;
use std::io::Read;
use url::Url;
use url::percent_encoding::percent_decode;
#[cfg(not(test))]
use urls::EVUrl;
use std::vec;
use std::str::FromStr;
use std::error::Error;
use cookie::CookieJar;

use std::iter::{Chain,Filter,FlatMap,Map};

use std::sync::mpsc::{channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::BufRead;

#[allow(unused_imports)]
use rustc_serialize::{json,Decodable,Decoder};

macro_rules! json_field {
    ($name:expr, $decoder:expr) => {
        try!($decoder.read_struct_field($name, 0, |d| Decodable::decode(d)))
    }
}

#[derive(Debug)]
pub enum EVError {
    Authentication(String),
    Decoder(String),
    IO(String),
    Http(String),
    Fetch,
    NotFound
}
impl fmt::Display for EVError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "foo")
    }
}
impl Error for EVError {
    fn description(&self) -> &str {
        "An error occurred"
    }
}
impl From<::hyper::error::Error> for EVError {
    fn from(e: ::hyper::error::Error) -> EVError {
        EVError::Http(e.to_string())
    }
}
impl From<::std::io::Error> for EVError {
    fn from(e: ::std::io::Error) -> EVError {
        EVError::IO(e.to_string())
    }
}
impl From<::rustc_serialize::json::DecoderError> for EVError {
    fn from(e: ::rustc_serialize::json::DecoderError) -> EVError {
        EVError::Decoder(e.to_string())
    }
}
impl From<::std::sync::mpsc::RecvError> for EVError {
    fn from(_: ::std::sync::mpsc::RecvError) -> EVError {
        EVError::Fetch
    }
}

/// Describes an id of an folder
#[derive(Copy, Clone, Debug)]
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

/// Describes a size of a folder
#[derive(Clone, Debug)]
pub enum FolderSize {
    Bytes(f32),
    Kilobytes(f32),
    Megabytes(f32),
    Gigabytes(f32),
}
impl FromStr for FolderSize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        words.next()
            .and_then(|val_str| str::parse::<f32>(val_str).map(|v| (v * 100.0).round() / 100.0).ok())
            .and_then(|val| {
                match words.next() {
                    Some("GB") => Some(FolderSize::Gigabytes(val)),
                    Some("MB") => Some(FolderSize::Megabytes(val)),
                    Some("KB") | Some("kB") => Some(FolderSize::Kilobytes(val)),
                    Some("B") => Some(FolderSize::Bytes(val)),
                    _ => None
                }
            })
            .ok_or("Not a valid size".into())
    }
}

/// Contains information of a folder
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FolderInfo {
    id: FolderId,
    pub name: String,
    pub size: FolderSize,
    pub has_unwatched: bool,
    pub has_wildcards: bool,
    pub has_pin: Option<i32>, // TODO: Is this correct? I have no idea.
    pub recordings_count: usize,
    session_headers: Option<Headers>,
}
impl Decodable for FolderInfo {
    fn decode<D : Decoder>(d: &mut D) -> Result<FolderInfo, D::Error> {
        d.read_struct("", 0, |d| {
            Ok(FolderInfo {
                id: {
                    match json_field!("id", d) {
                        0 => FolderId::Root,
                        n => FolderId::FolderId(n)
                    }
                },
                name: json_field!("name", d),
                size: {
                    let size_string: String = json_field!("size", d);
                    try!(FolderSize::from_str(&size_string[..]).map_err(|e| d.error(&e[..])))
                },
                has_unwatched: {
                    let b: String = json_field!("has_unwatched", d);
                    try!(str::parse::<bool>(&b[..]).map_err(|e| d.error(e.description())))
                },
                has_wildcards: {
                    let b: String = json_field!("has_wildcards", d);
                    try!(str::parse::<bool>(&b[..]).map_err(|e| d.error(e.description())))
                },
                has_pin: {
                    let b: String = json_field!("has_pin", d);
                    match b.len() {
                        0 => None,
                        _ => {
                            Some(try!(str::parse::<i32>(&b[..]).map_err(|e| d.error(e.description()))))
                        }
                    }
                },
                recordings_count: json_field!("recordings_count", d),
                session_headers: None,
            })
        })
    }
}

impl FolderInfo {
    fn root(rec_count: usize) -> FolderInfo {
        FolderInfo {
            id: FolderId::Root,
            name: "Root".into(),
            size: FolderSize::Bytes(0.0),
            has_unwatched: false,
            has_wildcards: false,
            has_pin: None,
            recordings_count: rec_count,
            session_headers: None,
        }
    }
}
impl Fetch for FolderInfo {
    type Output = Folder;
    fn fetch_into(self) -> Result<Folder, EVError> {
        self.fetch()
    }
    #[cfg(not(test))]
    fn fetch(&self) -> Result<Folder, EVError> {
        let url = EVUrl::Folder(self.id);
        let client = Client::new();
        let res = client.get(url).headers(self.session_headers.clone().unwrap()).send();
        res.map_err(EVError::from).and_then(|mut res| {
            let mut ok = String::new();
            try!(res.read_to_string(&mut ok));
            json::decode(&ok)
                .map_err(EVError::from)
                .and_then(|mut f: Folder| {
                    f.info = self.clone();
                    for finfo in f.folders.iter_mut() {
                        finfo.session_headers = self.session_headers.clone();
                    }
                    for rinfo in f.recordings.iter_mut() {
                        rinfo.session_headers = self.session_headers.clone();
                    }
                    Ok(f)
                })
        })
    }
    #[cfg(test)]
    fn fetch(&self) -> Result<Folder, EVError> {
        use std::io::BufReader;
        let file = File::open(format!("testdata/folder_{}.json", self.id)).unwrap();
        let line = BufReader::new(file).lines().next().unwrap().unwrap();
        let mut fldr: Folder = json::decode(&line).unwrap();
        fldr.info = self.clone();
        Ok(fldr)
    }
}

impl Fetch for RecordingInfo {
    type Output = Recording;
    fn fetch_into(self) -> Result<Recording, EVError> {
        self.fetch()
    }
    #[cfg(not(test))]
    fn fetch(&self) -> Result<Recording, EVError> {
        let url = EVUrl::Program(ProgramId::ProgramId(self.program_id));
        let client = Client::new();
        let res = client.get(url).headers(self.session_headers.clone().unwrap()).send();
        res.map_err(EVError::from)
            .and_then(|mut res| {
                let mut ok = String::new();
                try!(res.read_to_string(&mut ok).map_err(|e| EVError::from(e)));
                json::decode(&ok).map_err(EVError::from)
            })
    }

    #[cfg(test)]
    fn fetch(&self) -> Result<Recording, EVError> {
        use std::io::BufReader;
        let file = File::open(format!("testdata/recording_{}.json", self.program_id)).unwrap();
        let line = BufReader::new(file).lines().next().unwrap().unwrap();
        let mut rec: Recording = json::decode(&line).unwrap();
        rec.info = self.clone();
        Ok(rec)
    }
}

/// Folder in Elisa Viihde
#[allow(dead_code)]
#[derive(Debug)]
pub struct Folder {
    info: FolderInfo,
    folders: Vec<FolderInfo>,
    recordings: Vec<RecordingInfo>,
}

/// Folder's IntoIterator implementation iterates over all recordings
/// of all folders starting from and including the folder into_iter() is called to
impl IntoIterator for Folder {
    type Item = RecordingInfo;
    type IntoIter =
        Chain<
            FlatMap<
                Filter<
                    Map<
                        vec::IntoIter<FolderInfo>,
                        fn(FolderInfo) -> Result<Folder, EVError>
                    >,
                    fn(&Result<Folder, EVError>) -> bool
                >,
                vec::IntoIter<RecordingInfo>,
                fn(Result<Folder, EVError>) -> vec::IntoIter<RecordingInfo>
            >,
            vec::IntoIter<RecordingInfo>
        >;
    fn into_iter(self) -> Self::IntoIter {
        self.folders.into_iter()
            .map(<FolderInfo as Fetch>::fetch_into as fn(FolderInfo) -> Result<Folder, EVError>)
            .filter(Result::<Folder, EVError>::is_ok as fn(&Result<Folder, EVError>) -> bool)
            .flat_map(into_iter_unwrapper as fn(Result<Folder, EVError>) -> vec::IntoIter<RecordingInfo>)
            .chain(self.recordings.into_iter())
    }
}
fn into_iter_unwrapper(f: Result<Folder, EVError>) -> vec::IntoIter<RecordingInfo> {
    f.ok().unwrap().recordings.into_iter()
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
    /// Returns `Folders` over this folder
    pub fn folders(&'a self) -> Folders<'a> {
        Folders { index: 0, folder: self }
    }
    /// Returns `Recordings` over this folder
    pub fn recordings(&'a self) -> Recordings<'a> {
        Recordings { index: 0, folder: self }
    }
}

impl fmt::Display for Folder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.info.name)
    }
}

fn check_len_and_return_ref<T>(index: usize, vec: &Vec<T>) -> Option<(usize, &T)> {
    let count = vec.len();
    if count != 0 && index < count {
        Some((index + 1, &vec[index]))
    }
    else {
        None
    }
}

impl<'a> Iterator for Folders<'a> {
    type Item = &'a FolderInfo;
    fn next(&mut self) -> Option<Self::Item> {
        check_len_and_return_ref(self.index, &self.folder.folders)
            .and_then(|(idx, item)| {
                self.index = idx;
                Some(item)
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.folder.folders.len()))
    }
}

impl<'a> Iterator for Recordings<'a> {
    type Item = &'a RecordingInfo;
    fn next(&mut self) -> Option<Self::Item> {
        check_len_and_return_ref(self.index, &self.folder.recordings)
            .and_then(|(idx, item)| {
                self.index = idx;
                Some(item)
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.folder.recordings.len()))
    }
}

impl Folder {
    /// Fetches the root folder from Elisa Viihde
    /// The `CookieJar` needs to have a valid `SetCookie` that has the session token.
    /// You probably want to use `authentication::login` instead of this function.
    #[cfg(not(test))]
    pub fn fetch_root(jar: CookieJar) -> Result<Folder, EVError> {
        let headers = evimproved_headers(Some(jar));
        let client = Client::new();
        let ret = match client.get(EVUrl::Folder(FolderId::Root)).headers(headers.clone()).send() {
            Ok(mut res) => {
                let mut ok = String::new();
                try!(res.read_to_string(&mut ok));
                let mut folder: Folder = try!(json::decode(&ok));
                for finfo in folder.folders.iter_mut() {
                    finfo.session_headers = Some(headers.clone());
                }
                for rinfo in folder.recordings.iter_mut() {
                    rinfo.session_headers = Some(headers.clone());
                }
                Ok(folder)
            },
            Err(e) => Err(EVError::from(e))
        };
        ret
    }
    fn decode_folder<D : Decoder>(d: &mut D) -> Result<Folder, D::Error> {
        let recordings: Vec<RecordingInfo> = json_field!("recordings", d);
        Ok(Folder {
            info: FolderInfo::root(recordings.len()),
            folders: json_field!("folders", d),
            recordings: recordings,
        })
    }
    /// Recursively finds a folder under this folder with a name
    /// If multiple folders match, the returned folder is the first that was found
    pub fn find_by_name(&self, name: &str) -> Result<FolderInfo, EVError> {
        fn do_find(name: &str, folder: &Folder, found: Arc<AtomicBool>) -> Result<FolderInfo, EVError> {
            for finfo in folder.folders() {
                if &finfo.name == name {
                    found.store(true, Ordering::SeqCst);
                    return Ok(finfo.clone())
                }
            }
            let mut threads = Vec::with_capacity(folder.folders().size_hint().1.unwrap());
            for finfo in folder.folders() {
                let (tx, rx) = channel();
                let fi: FolderInfo = finfo.clone();
                let n = name.to_owned();
                let found = found.clone();
                let t = thread::spawn(move || {
                    let ret = match fi.fetch() {
                        Ok(fldr) => {
                            if found.load(Ordering::SeqCst) {
                                Err(EVError::NotFound)
                            }
                            else {
                                do_find(&n, &fldr, found)
                            }
                        }
                        Err(e) => Err(e)
                    };
                    let _ = tx.send(ret);
                });
                threads.push((t, rx));
            }
            for (thread, rx) in threads {
                match try!(rx.recv()) {
                    Ok(ret) => return Ok(ret),
                    _ => {}
                };
                let _ = thread.join();
            }
            Err(EVError::NotFound)
        }
        let found = Arc::new(AtomicBool::new(false));
        do_find(name, self, found)
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
    pub info: RecordingInfo,
    pub id: i32,
    pub name: String,
    pub channel: String,
    pub length: i32,
    pub start_time: String,
    pub end_time: String,
    pub url: Url,
    pub programviewid: i32,
    pub recordingid: i32
}

/// Contains information of a Recording
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RecordingInfo {
    pub id : i32,
    pub program_id : i32,
    pub folder_id : Option<i32>,
    pub name: String,
    pub channel: String,
    pub start_time: String, // TODO
    pub timestamp: String, // TODO
    pub viewcount: i32,
    pub length: i32,
    session_headers: Option<Headers>,
}

impl Default for RecordingInfo {
    fn default() -> RecordingInfo {
        RecordingInfo {
            id: 0,
            program_id: 0,
            folder_id: None,
            name: "".to_string(),
            channel: "".to_string(),
            start_time: "".to_string(),
            timestamp: "".to_string(),
            viewcount: 0,
            length: 0,
            session_headers: None,
        }
    }
}

impl Decodable for RecordingInfo {
    fn decode<D : Decoder>(d: &mut D) -> Result<RecordingInfo, D::Error> {
        d.read_struct("", 0, |d| {
            Ok(RecordingInfo {
                id: json_field!("id", d),
                program_id: json_field!("program_id", d),
                folder_id: {
                    let idstr: String = json_field!("folder_id", d);
                    str::parse::<i32>(&idstr).ok()
                },
                name: {
                    let percent_encoded_str : String = json_field!("name", d);
                    String::from_utf8(percent_decode(percent_encoded_str.as_bytes())).unwrap()
                },
                channel: json_field!("channel", d),
                start_time: json_field!("start_time", d),
                timestamp: json_field!("timestamp", d),
                viewcount: json_field!("viewcount", d),
                length: json_field!("length", d),
                session_headers: None
            })
        })
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

#[cfg(test)]
mod tests {
    use rustc_serialize::json;
    use std::io::BufReader;
    use super::{Recording, Folder, FolderId, FolderSize};
    use std::io::BufRead;
    use std::fs::File;

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
            assert!(f.folders().size_hint() == (0, Some(2)), "Folder iterator had invalid bounds");
            let mut has_items: bool = false;
            for fldr in f.folders() {
                has_items = true;
                match fldr.id {
                    FolderId::FolderId(1000001) | FolderId::FolderId(1000002) => {},
                    _ => assert!(false, "Folder id was invalid")
                }
                match &fldr.name[..] {
                    "Foldername" | "Test folder" => {},
                    _ => assert!(false, "Folder name was invalid")
                }
                println!("{:?}", fldr.size);
                match fldr.size {
                    FolderSize::Gigabytes(57.31) | FolderSize::Gigabytes(865.87) => {},
                    _ => assert!(false, "Invalid size")
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

    #[test]
    fn able_to_find_a_folder_by_name() {
        setup_test!("testdata/root_folder.json", |f : Folder| {
            let f1 = f.find_by_name("Test folder");
            assert!(f1.is_ok());
            let f2 = f.find_by_name("Does not exist");
            assert!(f2.is_err());
        });
    }
}

