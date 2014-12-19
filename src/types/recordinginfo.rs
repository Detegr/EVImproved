use types::percentencodedstring::PercentEncodedString;
use url::Url;

#[allow(dead_code)]
#[deriving(Decodable)]
pub struct RecordingInfo {
    id : int,
    name : PercentEncodedString,
    channel : String,
    length : int,
    start_time : String,
    end_time : String,
    url : Url,
    programviewid : int,
    recordingid : int
}

#[allow(dead_code)]
impl RecordingInfo {
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
