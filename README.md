# EVImproved
[![Build Status](https://travis-ci.org/Detegr/EVImproved.svg?branch=master)](https://travis-ci.org/Detegr/EVImproved)

Reimplementation of my (unreleased, because it's awful) 'Elisa Viihde Improved' Haskell program that moves duplicate recordings to a user-defined folder. However, this time I'm more focusing on making a Rust-library that allows people to command Elisa Viihde with a sane API rather than doing an hacky application for my own use.

## Example
Fetch names of folders in the root folder
```rust
extern crate evimproved;
use evimproved::authentication::login;
fn main() {
  let root = login("username", "password").unwrap();
  for folder in root.folders() {
    println!("{}", folder.name);
  }
}
```
