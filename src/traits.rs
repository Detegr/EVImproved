use types::EVError;

pub trait Fetch {
    type Output;
    fn fetch_into(self) -> Result<Self::Output, EVError>;
    fn fetch(&self) -> Result<Self::Output, EVError>;
}
