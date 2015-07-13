# EVImproved
[![Build Status](https://travis-ci.org/Detegr/EVImproved.svg?branch=master)](https://travis-ci.org/Detegr/EVImproved)

Reimplementation of my (unreleased, because it's awful) 'Elisa Viihde Improved' Haskell program that moves duplicate recordings to a user-defined folder. However, this time I'm more focusing on making a Rust-library that allows people to command Elisa Viihde with a sane API rather than doing an hacky application for my own use.

## Example
```rust
extern crate evimproved;
use evimproved::authentication;

fn main() {
    let root = authentication::login("username", "password").unwrap();

    // Iteration over all folders
    for folder in root.folders() {
        println!("{}", folder.name);
    }

    // Iteration over recordings in a certain folder
    for recording in root.recordings() {
        println!("{}", recording.name);
    }

    // Flat iteration over all recordings in Elisa Viihde
    for recording in root {
        println!("{}", recording.name);
    }

}
```
