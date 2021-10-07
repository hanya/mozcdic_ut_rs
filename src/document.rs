
// Ported from update_documents.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use regex::*;

use super::util::*;


pub fn update_documents() -> std::io::Result<()> {
    const README_NAME: &str = "../README.md";

    // Get date, mozcver, utdicdate, sha256sum

    // Get date
    let date = command_wait_output("date", vec!["+\"%Y-%m-%d\""])?;

    // Get mozcver
    let mozcver = {
        let re = Regex::new(r"mozc-([\d.]*?).102.tar.bz2").unwrap();
        let mut ver = String::new();
        for path in MatchedFiles::new(".", r"mozc-([\d.]*?).102.tar.bz2")? {
            if let Some(captures) = re.captures(&path) {
                ver.push_str(captures.get(1).unwrap().as_str());
                break;
            }
        }
        ver
    };
    println!("mozcver = {}", mozcver);

    // Get utdicdate
    let utdicdate = {
        let re = Regex::new(r#"UTDICDATE="(\d*)""#).unwrap();
        let s = read_file("../src/make-dictionaries.sh")?;
        if let Some(captures) = re.captures(&s) {
            String::from(captures.get(1).unwrap().as_str())
        } else {
            String::from("")
        }
    };
    println!("utdicdate = {}", utdicdate);

    // Get sha256sum of Mozc
    let sha256 = {
        let arg = format!("mozc-{}.tar.bz2", mozcver);
        let v = command_wait_output("sha256sum", vec![arg.as_str()])?;
        String::from(v.split(" ").next().unwrap())
    };
    println!("mozc sha256sum = {}", sha256);


    // Update README.md
    {
        let mut lines = read_file(README_NAME)?;

        // todo
        let re_date = Regex::new(r"date: \d{4}-\d{2}-\d{2}").unwrap();
        let re_mozcver = Regex::new(r"mozc-\d\.\d{2}\.\d{4}\.102").unwrap();
        let re_utdicdate = Regex::new(r"mozcdic-ut-\d{8}").unwrap();
        let re_fcitx5 = Regex::new(r"fcitx5-mozc-ut-\d{8}").unwrap();

        lines = re_date.replace_all(&lines, &format!("date: {}", date)).to_string();
        lines = re_mozcver.replace_all(&lines, &format!("mozc-{}", mozcver)).to_string();
        lines = re_utdicdate.replace_all(&lines, &format!("fcitx5-mozc-ut-{}", &utdicdate)).to_string();
        lines = re_fcitx5.replace_all(&lines, &format!("fcitx5-mozc-ut-{}", &utdicdate)).to_string();

        let f = File::create(README_NAME)?;
        let mut writer = BufWriter::new(f);
        writer.write(lines.as_bytes())?;
    }

    // Update PKGBUILD
    let mut pkgbuild = Vec::new();
    for path in MatchedFiles::new("../pkgbuild", r"^.*\.PKGBUILD$")? {
        pkgbuild.push(path);
    }

    let re_mozcver = Regex::new(r"_mozcver=\d\.\d{2}\.\d{4}\.102").unwrap();
    let re_utdicver = Regex::new(r"_utdicver=\d{8}").unwrap();
    let re_sha256 = Regex::new(r"  \'.{64}\'\n").unwrap();

    for name in pkgbuild {
        let mut lines = read_file(&name)?;

        // Update filenames
        let newfile = format!("{}-{}.PKGBUILD", name.split_once("-2").unwrap().0, &utdicdate);

        std::fs::rename(name, &newfile)?;

        // Update mozcver
        lines = re_mozcver.replace(&lines, &format!(r"_mozcver={}", mozcver)).to_string();

        // Update utdicver
        lines = re_utdicver.replace(&lines, &format!(r"_utdicver={}", utdicdate)).to_string();

        // Update sha256sum
        lines = re_sha256.replace(&lines, &format!(r"  '{}'\n", sha256)).to_string();

        let f = File::create(&newfile)?;
        let mut writer = BufWriter::new(f);
        writer.write(lines.as_bytes())?;
    }

    Ok(())
}
