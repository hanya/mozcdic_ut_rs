

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


fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let mut i = 1;
    let count = args.len();
    if count <= 1 || &args[1] == "help" {
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
        return;
    }
    while i < count {
        let argument = &args[i];
        let r = match argument.as_str() {
            "altcannadic" => run_convert_alt_cannadic_to_mozcdic(),
            "chimei" => run_fix_ken_all()
                .and_then(|_| generate_chimei_for_mozcdic()),
            "clean" => util::command_wait("rm", vec!["-rf", "build"]),
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
