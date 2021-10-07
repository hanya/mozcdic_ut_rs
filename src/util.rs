

use std::process::Command;
use std::io::{BufReader, Read};
use std::fs::File;
use std::path::PathBuf;

use regex::*;


/// Returns files matched to regex pattern in the specified directory.
#[derive(Debug)]
pub struct MatchedFiles {
    read_dir: std::fs::ReadDir,
    dirpath: String,
    pattern: Regex,
}

impl MatchedFiles {
    /// Creates instance from directory path and pattern.
    pub fn new(dirpath: &str, pattern: &str) -> std::io::Result<MatchedFiles> {
        Ok(MatchedFiles {
            read_dir: std::fs::read_dir(dirpath)?,
            dirpath: String::from(dirpath),
            pattern: Regex::new(pattern).unwrap(),
        })
    }
}

impl Iterator for MatchedFiles {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Ok(entry)) = self.read_dir.next() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.to_str() {
                    if self.pattern.is_match(name) {
                        let mut path = PathBuf::from(&self.dirpath);
                        path.push(name);
                        return path.as_path()
                                   .to_str()
                                   .map_or(None, |s| Some(String::from(s)));
                    }
                }
            }
        }
        None
    }
}

/// Removes files which match to regex pattern from specified directory.
pub fn remove_matched(dirpath: &str, pattern: &str) -> std::io::Result<()> {
    for path in MatchedFiles::new(dirpath, pattern)? {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Reads UTF-8 file content as string.
pub fn read_file(filename: &str) -> std::io::Result<String> {
    let mut s = String::new();
    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut s)?;
    Ok(s)
}

/// Reads data from file as bytes.
pub fn read_file_vec(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Execute command with arguments and wait until finished.
pub fn command_wait(cmd: &str, args: Vec<&str>) -> std::io::Result<()> {
    if let Ok(mut child) = Command::new(cmd).args(args).spawn() {
        child.wait()?;
    }
    Ok(())
}

/// Execute command with arguments and wait until finished, output is returned.
pub fn command_wait_output(cmd: &str, args: Vec<&str>) -> std::io::Result<String> {
    let output = Command::new(cmd).args(args).output()?;
    Ok(unsafe { String::from_utf8_unchecked(output.stdout) })
}

/// Returns number of cores in the CPU.
pub fn get_core_count() -> std::io::Result<usize> {
    let re = Regex::new(r"^cpu cores\s*: (\d*)").unwrap();
    let info = command_wait_output("grep", vec!["cpu.cores", "/proc/cpuinfo"])?;
    if let Some(captures) = re.captures(&info) {
        Ok(usize::from_str_radix(captures.get(1).unwrap().as_str(), 10).unwrap() - 1)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "failed to obtain core count from cpuinfo"))
    }
}

/// Converts from katakana to hiragana.
/// Returns None if the value is out of katakana.
#[inline]
pub fn katakana_to_hiragana(c: char) -> Option<char> {
    match c {
        'ァ'..='ヶ' | 'ヽ' | 'ヾ' => char::from_u32(c as u32 - ('ァ' as u32 - 'ぁ' as u32)),
        _ => None,
    }
}

/// Converts from katakana to hiragana.
/// Additionally replaces from ゐ/ゑ to い/え.
pub fn to_hiragana_replace_ie(s: &str) -> String {
    let mut ns = String::with_capacity(s.len());
    s.chars().for_each(|c| {
        let c = katakana_to_hiragana(c).unwrap_or(c);
        let c = match c {
            'ゐ' => 'い',
            'ゑ' => 'え',
            _ => c,
        };
        ns.push(c);
    });
    ns
}

/// Converts from fullwidth ASCII to halfwidth ASCII character.
/// Returns None if the value is out of ASCII range.
#[inline]
pub fn ascii_fullwidth_to_halfwidth(c: char) -> Option<char> {
    match c {
        '！'..='～' => char::from_u32(c as u32 - ('！' as u32 - '!' as u32)),
        _ => None,
    }
}

/// Converts from fullwidth ASCII to halfwidth ASCII character.
pub fn ascii_to_halfwidth(s: &str) -> Option<String> {
    // No need to convert in most cases, so just check and convert if required.
    if s.chars().any(|c| '！' <= c && c <= '～') {
        let mut ns = String::with_capacity(s.len());
        s.chars().for_each(|c| ns.push(ascii_fullwidth_to_halfwidth(c).unwrap_or(c)));
        Some(ns)
    } else {
        None
    }
}

/// Conversion map from halfwidth katakana to fullwidth hirakana.
const HALFWIDTH_KATAKANA_TO_FULLWIDTH_HIRAKANA: [char; 56] = [
    //'・', // removed in half_to_hiragana_no_dot
    'を','ぁ','ぃ','ぅ','ぇ','ぉ','ゃ','ゅ','ょ','っ','ー',
    'あ','い','う','え','お','か','き','く','け','こ',
    'さ','し','す','せ','そ','た','ち','つ','て','と',
    'な','に','ぬ','ね','の','は','ひ','ふ','へ','ほ',
    'ま','み','む','め','も','や','ゆ','よ',
    'ら','り','る','れ','ろ','わ','ん',
];

/// Convert char of halfwidth katakana to hiragana.
#[inline]
fn half_to_hiragana(c: char) -> char {
    let index = (c as u32 - 'ｦ' as u32) as usize;
    HALFWIDTH_KATAKANA_TO_FULLWIDTH_HIRAKANA[index]
}

/// Halfwidth katakana to hiragana conversion with dot removal.
pub fn half_to_hiragana_no_dot(s: &str) -> String {
    let mut ss = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '･' => {}, // remove dot here
            'ｦ'..='ﾝ' => {
                // ユニコードでは、「へべぺ」のように連続
                if it.peek() == Some(&'ﾞ') {
                    let cb = half_to_hiragana(c);
                    if cb == 'う' {
                        ss.push('ゔ');
                    } else {
                        // 濁点付きは +1
                        ss.push(char::from_u32(cb as u32 + 1).unwrap());
                    }
                    it.next(); // consume dakuten
                } else if it.peek() == Some(&'ﾟ') {
                    let cb = half_to_hiragana(c);
                    // 半濁点付きは + 2
                    ss.push(char::from_u32(cb as u32 + 2).unwrap());
                    it.next(); // consume handakuten
                } else {
                    ss.push(half_to_hiragana(c));
                }
            },
            _ => ss.push(c),
        }
    }
    ss
}
