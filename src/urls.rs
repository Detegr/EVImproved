/* vim: set et: */

use types::FolderId;
use types::ProgramId;
use std::fmt;

pub enum EVUrl {
    Login,
    Folder(FolderId),
    Program(ProgramId),
    Move(ProgramId, FolderId)
}
impl fmt::Show for EVUrl {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EVUrl::Login => write!(fmt, "https//api.elisaviihde.fi/etvrecorder/login.sl"),
            EVUrl::Folder(ref id) => match *id {
                FolderId::Root => write!(fmt, "https://api.elisaviihde.fi/etvrecorder/ready.sl?ajax=true"),
                ref id => write!(fmt, "https://api.elisaviihde.fi/etvrecorder/ready.sl?folderid={}&ppos=0&ajax=true", id),
            },
            EVUrl::Program(ref id) => write!(fmt, "https://api.elisaviihde.fi/etvrecorder/program.sl?programid={}&ppos=0&ajax=true", id),
            EVUrl::Move(ref pid, ref fid) => write!(fmt, "https://api.elisaviihde.fi/etvrecorder/ready.sl?ajax=true&move=true&destination={}&programviewid={}", fid, pid)
        }
    }
}

mod tests {
    use super::EVUrl;
    use types::FolderId;
    use types::ProgramId;

    #[test]
    fn show_login_url() {
        let url = EVUrl::Login;
        assert!(url.to_string() == "https//api.elisaviihde.fi/etvrecorder/login.sl");
    }

    #[test]
    fn show_root_folder_url() {
        let url = EVUrl::Folder(FolderId::Root);
        assert!(url.to_string() == "https://api.elisaviihde.fi/etvrecorder/ready.sl?ajax=true");
    }

    #[test]
    fn show_non_root_folder_url() {
        let url = EVUrl::Folder(FolderId::FolderId(123));
        assert!(url.to_string() == "https://api.elisaviihde.fi/etvrecorder/ready.sl?folderid=123&ppos=0&ajax=true");
    }

    #[test]
    fn show_program_url() {
        let url = EVUrl::Program(ProgramId::ProgramId(123));
        assert!(url.to_string() == "https://api.elisaviihde.fi/etvrecorder/program.sl?programid=123&ppos=0&ajax=true");
    }

    #[test]
    fn show_move_url() {
        let url = EVUrl::Move(ProgramId::ProgramId(123), FolderId::FolderId(321));
        assert!(url.to_string() == "https://api.elisaviihde.fi/etvrecorder/ready.sl?ajax=true&move=true&destination=321&programviewid=123");
    }
}
