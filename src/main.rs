

use std::fs::File;
use std::io::Write;

mod alt_cannadic;
mod chimei;
mod costs;
mod document;
mod edict2;
mod filter;
mod jawiki_article;
mod jawiki_titles;
mod jinmei_ut;
mod mozc;
mod neologd;
mod skkdic;
mod sudachidict;
mod util;

use alt_cannadic::*;
use chimei::*;
use costs::*;
use document::*;
use edict2::*;
use filter::*;
use jawiki_article::*;
use jawiki_titles::*;
use jinmei_ut::*;
use mozc::*;
use neologd::*;
use skkdic::*;
use sudachidict::*;
use util::*;


/// Shows help message.
fn help() {
    print!("usage: cargo run --release COMMAND [FILENAME]

COMMAND:
  altcannadic      converts alt-canna dictionary
  chimei           generates chimei dictionary from zipcode list
  edict2           converts edict2 dictionary
  jawikiarticles   read jawiki articles and generate dictionary
  jawikititles     read jawiki titles for calculating costs later
  jinmeiut         converts jinmei ut dictionary
  neologd          converts mecab dictionary
  skkdic           converts SKK dictionary
  sudachidic       converts sudachidic dictionary

  mozc             gets mozc source and required id.def file
                   required by all commands generates dictionary

  costs            recalculates cost, needs FILENAME
  unnecessary      removes unnecessary entries from dictionary
                   needs FILENAME
  unsuitable       removes unsuitable entries from dictionary
                   needs FILENAME

  document         update document

  clean            removes build directory
  help             this message
");
}

/// Execute command.
fn command_runner(args: &Vec<&str>) {
    let mut i = 0;
    let count = args.len();
    while i < count {
        let argument = args[i];
        let r = match argument {
            "altcannadic" => run_convert_alt_cannadic_to_mozcdic(),
            "chimei" => run_fix_ken_all()
                .and_then(|_| generate_chimei_for_mozcdic()),
            "clean" => command_wait("rm", vec!["-rf", "build"]),
            "costs" => {
                // costs filename
                if i + 1 < count {
                    let filename = &args[i + 1];
                    i += 1;
                    let dicname = format!("{}.costs", filename);
                    calculate_costs(filename, &dicname)
                } else {
                    println!("costs option requires file path");
                    std::process::exit(-1);
                }
            },
            "document" => update_documents(),
            "edict2" => run_convert_edict2_to_mozcdic(),
            "jawikiarticles" => run_generate_jawiki_ut(),
            "jawikititles" => run_add_search_results_to_each_title(),
            "jinmeiut" => run_convert_jinmei_ut_to_mozcdic(),
            "help" => continue,
            "mozc" => get_the_latest_mozc(),
            "neologd" => run_convert_neologd_to_mozcdic(),
            "skkdic" => run_convert_skkdic_to_mozcdic(),
            "sudachidict" => run_convert_sudachidict_to_mozcdic(),
            "unnecessary" => {
                // unnecessary filename
                if i + 1 < count {
                    let filename = &args[i + 1];
                    i += 1;
                    let dicname = format!("{}.need", filename);
                    remove_unnecessary_entries(filename, &dicname)
                } else {
                    println!("unnecessary option requires file path");
                    std::process::exit(-1);
                }
            },
            "unsuitable" => {
                // unsuitable filename
                if i + 1 < count {
                    let filename = &args[i + 1];
                    i += 1;
                    filter_unsuitable_entries(filename, filename)
                } else {
                    println!("unsuitable option requires file path");
                    std::process::exit(-1);
                }
            },
            _ => {
                println!("unknown option: {}", &argument);
                std::process::exit(-1);
            },
        };
        if let Err(e) = r {
            println!("{}", e);
            std::process::exit(-1);
        }
        i += 1;
    }
}

#[derive(Debug, Default)]
struct Licenses {
    apl2: bool,
    ccbysa3: bool,
    gpl2: bool,
}

/// Run to make dictionary.
fn workflow(args: &Vec<&str>) {
    let mut licenses = Licenses::default();
    let mut readme = Vec::new();

    command_wait("mkdir", vec!["build"]).unwrap();
    std::env::set_current_dir("build").unwrap();

    command_wait("rm", vec!["mozcdic-*"]).unwrap();
    command_wait("rm", vec!["jawiki-ut-*"]).unwrap();

    command_runner(&vec!["mozc"]);
    command_runner(&vec!["jawikititles"]);

    let status = {
        let mut status: u8 = 0;
        let mut i = 1; // skip workflow
        let count = args.len();
        while i < count {
            let argument = args[i];
            match argument {
                "altcannadic" => {
                    status |= 0x1;
                    command_runner(&vec!["altcannadic"]);
                    licenses.gpl2 = true;
                    readme.push(String::from(
                        "* Entries came from alt-cannadic are licensed under General Public License 2.0.
  https://ja.osdn.net/projects/alt-cannadic/"));
                }
                "chimei" => {
                    status |= 0x2;
                    command_runner(&vec!["chimei"]);
                    readme.push(String::from(
                        "* Location data is came from zipcode data made by Japan post, licensed under public domain.
  https://www.post.japanpost.jp/zipcode/dl/readme.html"));
                }
                "edict2" => {
                    status |= 0x4;
                    command_runner(&vec!["edict2"]);
                    licenses.ccbysa3 = true;
                    readme.push(String::from(
                        "* Entries from edict2 are licensed under CC-BY-SA 3.0.
  http://ftp.edrdg.org/pub/Nihongo/"));
                }
                "jawikiarticles" => {
                    status |= 0x8;
                    command_runner(&vec!["jawikiarticles"]);
                    command_runner(&vec!["unsuitable", "mozcdic-ut-jawiki.txt"]);
                    licenses.ccbysa3 = true;
                    readme.push(String::from(
                        "* Entries from ja.wikipedia are licensed under CC-BY-SA 3.0.
  https://ja.wikipedia.org/wiki/Wikipedia:%E3%83%87%E3%83%BC%E3%82%BF%E3%83%99%E3%83%BC%E3%82%B9%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%89"));
                }
                "jinmeiut" => {
                    status |= 0x10;
                    command_runner(&vec!["jinmeiut"]);
                    licenses.apl2 = true;
                    readme.push(String::from(
                        "* Entries from jinmeiut are licensed under Apache License 2.0.
  http://linuxplayers.g1.xrea.com/mozc-ut.html"));
                }
                "neologd" => {
                    status |= 0x20;
                    command_runner(&vec!["neologd"]);
                    command_runner(&vec!["unsuitable", "mozcdic-ut-neologd.txt"]);
                    licenses.apl2 = true;
                    readme.push(String::from(
                        "* Entries from neologd are licensed under Apache License 2.0.
  https://github.com/neologd/mecab-ipadic-neologd"));
                }
                "skkdic" => {
                    status |= 0x40;
                    command_runner(&vec!["skkdic"]);
                    licenses.gpl2 = true;
                    readme.push(String::from(
                        "* Entries from skkdic are licensed under General Public License 2.0.
  http://openlab.jp/skk"));
                }
                "sudachidic" => {
                    status |= 0x80;
                    command_runner(&vec!["sudachidic"]);
                    command_runner(&vec!["unsuitable", "mozcdic-ut-sudachidict-core.txt"]);
                    command_runner(&vec!["unsuitable", "mozcdic-ut-sudachidict-notcore.txt"]);
                    licenses.apl2 = true;
                    readme.push(String::from(
                        "* Entries from SudachiDict are licensed under Apache License 2.0.
  https://github.com/WorksApplications/SudachiDict"));
                }
                _ => {
                    println!("unknown option: {}", argument);
                    std::process::exit(-1);
                }
            }
            i += 1;
        }
        status
    };
    println!("dictionary status: {}", status);

    let dicname = "mozcdic-ut.txt";
    let dicname_pre = "mozcdic-ut-pre.txt";
    {
        let mut f = File::create(&dicname_pre).unwrap();
        let mut append = |filename: &str| {
            let mut fc = File::open(filename).unwrap();
            std::io::copy(&mut fc, &mut f).expect(filename);
        };

        let mut i = 1;
        let count = args.len();
        while i < count {
            let argument = args[i];
            match argument {
                "altcannadic" => {
                    append("mozcdic-ut-alt-cannadic.txt");
                    append("mozcdic-ut-alt-cannadic-jinmei.txt");
                }
                "chimei" => {
                    append("mozcdic-ut-chimei.txt");
                }
                "edict2" => {
                    append("mozcdic-ut-edict2.txt");
                }
                "jawikiarticles" => {
                    append("mozcdic-ut-jawiki.txt");
                }
                "jinmeiut" => {
                    append("mozcdic-ut-jinmei.txt");
                }
                "neologd" => {
                    append("mozcdic-ut-neologd.txt");
                }
                "skkdic" => {
                    append("mozcdic-ut-skkdic.txt");
                }
                "sudachidic" => {
                    append("mozcdic-ut-sudachidict-core.txt");
                    append("mozcdic-ut-sudachidict-notcore.txt");
                }
                _ => {
                    println!("unknown option: {}", argument);
                    std::process::exit(-1);
                }
            }
            i += 1;
        }
    }

    command_runner(&vec!["unnecessary", &dicname_pre]);
    let dicname_need = format!("{}.need", &dicname_pre);
    command_runner(&vec!["cost"]);
    let dicname_costs = format!("{}.costs", &dicname_need);
    command_wait("mv", vec![&dicname_costs, &dicname]).unwrap();

    // Generates README.md file.
    {
        let head = "mozcdic_ut dictionary generated by mozcdic_ut_rs which
is ported from original mozcdic_ut to Rust.

Before building mozc, merge the dictionary into mozc oss dictionary as follows.

```
cat mozcdic-ut-XX.txt >> mozc-master/src/data/dictionary_oss/dictionary00.txt
```

This dictionary contains entries from the following projects.\n";
        let mut f = File::create("README.md").unwrap();
        f.write_all(head.as_bytes()).unwrap();
        for line in readme {
            f.write_all(line.as_bytes()).unwrap();
            f.write(b"\n").unwrap();
        }
    }

    // Generates archive
    let archive_name = "mozcdic-ut.tar.bz2";
    let mut args = vec!["-cfj", &archive_name, dicname];

    if licenses.apl2 {
        std::fs::copy("../data/license/Apache-2.0.txt", "Apache-2.0.txt").unwrap();
        args.push("Apache-2.0.txt");
    }
    if licenses.ccbysa3 {
        std::fs::copy("../data/license/CC-BY-SA-3.0.txt", "CC-BY-SA-3.0.txt").unwrap();
        args.push("CC-BY-SA-3.0.txt");
    }
    if licenses.gpl2 {
        std::fs::copy("../data/license/GPL-2.0.txt", "GPL-2.0.txt").unwrap();
        args.push("GPL-2.0.txt");
    }

    command_wait("tar", args).unwrap();
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        let a = args
            .iter()
            .skip(1)
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        match args[1].as_str() {
            "workflow" => workflow(&a),
            "help" => help(),
            _ => command_runner(&a),
        }
    } else {
        help()
    }
}
