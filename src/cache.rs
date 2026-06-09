#![allow(dead_code)]
use std::path::PathBuf;
use std::fs;
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn cache_path(key: &str) -> PathBuf {
    let mut dir = dirs::home_dir().expect("no home dir");
    dir.push(".c-dsl");
    dir.push("cache");
    fs::create_dir_all(&dir).ok();
    dir.push(format!("{}.json", STANDARD.encode(key)));
    dir
}

pub fn get_cached(key: &str) -> Option<String> {
    let path = cache_path(key);
    fs::read_to_string(path).ok()
}

pub fn set_cached(key: &str, value: &str) {
    let path = cache_path(key);
    fs::write(path, value).ok();
}
