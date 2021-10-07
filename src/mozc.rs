
// Ported from get_the_latest_mozc.rb file.

use std::io::{BufRead, BufReader};
use std::fs::File;

use regex::*;

use super::util::*;


/// Get Mozc id matches to passed regexp,
/// ex. r"(\d*) 名詞,固有名詞,地域,一般,\*,\*,\*".
pub fn get_id(exp: &str) -> std::io::Result<String> {
    let re = Regex::new(exp).unwrap();

    let s = read_file("id.def").expect("id.def file not found");
    if let Some(caps) = re.captures(&s) {
        Ok(String::from(caps.get(1).unwrap().as_str()))
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "id not found"));
    }
}

pub fn parse_mozc_version_template() -> std::io::Result<String> {
    let f = File::open("mozc_version_template.bzl")?;
    let mut reader = BufReader::new(f);

    let mut version = String::with_capacity(32);
    let mut line = String::new();
    while let Ok(1..) = reader.read_line(&mut line) {
        if let Some(index) = line.find("MAJOR = ") {
            version.push_str(unsafe { line.get_unchecked(index + 8..) }.trim_end());
            version.push('.');
            line.clear();
            continue;
        }
        if let Some(index) = line.find("MINOR = ") {
            version.push_str(unsafe { line.get_unchecked(index + 8..) }.trim_end());
            version.push('.');
            line.clear();
            continue;
        }
        if let Some(index) = line.find("BUILD = ") {
            version.push_str(unsafe { line.get_unchecked(index + 8..) }.trim_end());
            line.clear();
            continue;
        }
    }

    Ok(version)
}

pub fn get_mozc(version: &str) -> std::io::Result<()> {
    let mozcdir = format!("mozc-{}.102", version);

    // Get the latest mozc
    let tarfile = format!("{}.tar.bz2", mozcdir);
    if let Ok(_) = File::open(&tarfile) {
        println!("{} already exists.", tarfile);
        return Ok(());
    }

    let zipfile = format!("{}.zip", mozcdir);
    if let Ok(_) = File::open(&zipfile) {
        println!("{}.zip already exists.", zipfile);
    } else {
        command_wait("rm", vec!["-f", "mozc-*.zip"])?;
        command_wait("wget", vec!["https://github.com/google/mozc/archive/refs/heads/master.zip", "-O", &zipfile])?;
    }

    command_wait("rm", vec!["-rf", "mozc-master"])?;
    command_wait("unzip", vec!["-qq", &zipfile])?;
    command_wait("cp", vec!["mozc-master/src/data/dictionary_oss/id.def", "."])?;

    {
        let mut f = File::create("mozcdic.txt")?;
        for i in 0..10 {
            let path = format!("mozc-master/src/data/dictionary_oss/dictionary{:02}.txt", i);
            if let Ok(mut fd) = File::open(path) {
                std::io::copy(&mut fd, &mut f)?;
            }
        }
    }

    println!("Compress {}...", mozcdir);
    command_wait("rm", vec!["-f", "mozc-*.tar.bz2"])?;
    command_wait("rm", vec!["-rf", "mozc-master/src/third_party/"])?;
    command_wait("mv", vec!["mozc-master", &mozcdir])?;
    command_wait("tar", vec!["-cjf", &tarfile, &mozcdir])?;
    command_wait("rm", vec!["-rf", &mozcdir])?;

    return Ok(());
}

pub fn get_the_latest_mozc() -> std::io::Result<()> {
    command_wait("wget", vec!["-N", "https://raw.githubusercontent.com/google/mozc/master/src/data/version/mozc_version_template.bzl"])?;
    let version = parse_mozc_version_template()?;
    get_mozc(&version)?;

    return Ok(());
}

