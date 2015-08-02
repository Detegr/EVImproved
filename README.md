# EVImproved
[![Build Status](https://travis-ci.org/Detegr/EVImproved.svg?branch=master)](https://travis-ci.org/Detegr/EVImproved)

Reimplementation of my (unreleased, because it's awful) 'Elisa Viihde Improved' Haskell program that moves duplicate recordings to a user-defined folder. However, this time I'm more focusing on making a Rust-library that allows people to command Elisa Viihde with a sane API rather than doing an hacky application for my own use.

## Example
```rust
extern crate evimproved;                                                                                                                                                                                                           
use evimproved::authentication::login;                                                                                                                                                                                             
use evimproved::traits::Fetch;                                                                                                                                                                                                     
use evimproved::types::EVError;                                                                                                                                                                                                    
use std::error::Error;                                                                                                                                                                                                             
                                                                                                                                                                                                                                   
fn main() {                                                                                                                                                                                                                        
    let root = login("username", "password").unwrap();                                                                                                                                                                             
                                                                                                                                                                                                                                   
    // Iteration over folders in a certain folder                                                                                                                                                                                  
    for folder in root.folders() {                                                                                                                                                                                                 
        println!("{}", folder.name);                                                                                                                                                                                               
    }                                                                                                                                                                                                                              
                                                                                                                                                                                                                                   
    // Collect a Vec of recording names                                                                                                                                                                                            
    // from a folder called "Movies" (if it exists)                                                                                                                                                                                
    let movie_names = root.find_by_name("Movies")                                                                                                                                                                                  
        .and_then(Fetch::fetch_into)                                                                                                                                                                                               
        .and_then(|movies_folder| {                                                                                                                                                                                                
            Ok(movies_folder.recordings()                                                                                                                                                                                          
                .map(|r| r.name.to_owned())                                                                                                                                                                                        
                .collect::<Vec<String>>())                                                                                                                                                                                         
        });                                                                                                                                                                                                                        
                                                                                                                                                                                                                                   
    match movie_names {                                                                                                                                                                                                            
        Ok(movies) => {                                                                                                                                                                                                            
            for movie in movies {                                                                                                                                                                                                  
                println!("{}", movie);                                                                                                                                                                                             
            }                                                                                                                                                                                                                      
        }                                                                                                                                                                                                                          
        Err(EVError::NotFound) => println!("Folder not found"),                                                                                                                                                                    
        Err(e) => println!("{}", e.description())                                                                                                                                                                                  
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
