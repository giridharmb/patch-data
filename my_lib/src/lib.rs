#[macro_use]
extern crate lazy_static;
use lazy_static::lazy_static;

use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;


pub fn do_something() {
    println!("do_something()...");
}