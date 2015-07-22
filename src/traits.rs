pub trait Fetch {
    type Output;
    fn fetch_into(self) -> Option<Self::Output>;
    fn fetch(&self) -> Option<Self::Output>;
}
