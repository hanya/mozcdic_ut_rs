
// Ported from convert_alt_cannadic_to_mozcdic.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use encoding_rs::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


#[derive(Clone)]
struct Entry {
    key: String,
    yomi: String,
    hyouki: String,
    cost: String,
}

fn convert_alt_cannadic_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Mozcの品詞IDを取得
    // 「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    // 「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,一般,\*,\*,\*,\*,\*")?;

    let buf = read_file_vec(filename)?;
    let mut l2 = Vec::new();
    let (cow, _encoding_used, _had_errors) = EUC_JP.decode(&buf);
    for line in cow.lines() {
        // あきびん #T35*202 空き瓶 空瓶 #T35*151 空きビン 空ビン #T35*150 空きびん

        let mut s = line.trim_end().split(" ");
        
        let mut yomi = String::from(s.next().unwrap());
        yomi = yomi.replace("う゛", "ゔ");

        // 読みがひらがな以外を含む場合はスキップ
        if yomi.chars()
            .any(|c| !(('ぁ' <= c && c <= 'ゔ') || c == 'ー')) {
            continue;
        }

        let mut hinshi = "";

        while let Some(entry) = s.next() {
            // cannadicの品詞を取得
            if entry.starts_with("#") {
                hinshi = entry;
                continue;
            }

            let hyouki = entry;

            // cost を作成
            // alt-cannadicのコストは大きいほど優先度が高い。
            let cost = if let Some((_, base_cost)) = hinshi.split_once('*') {
                7000 - i32::from_str_radix(base_cost, 10).unwrap()
            } else {
                continue;
            };

            // 収録する品詞を選択
            if let Some("#T3" | "#T0" | "#JN" | "#KK" | "#CN") = hinshi.get(0..3) {
                l2.push(Entry {
                    key: format!("{}\t{}\t{}\n", &yomi, hyouki, cost),
                    yomi: String::from(&yomi),
                    hyouki: String::from(hyouki),
                    cost: cost.to_string(),
                });
            }
        }
    }

    let mut lines = l2;
    lines.par_sort_unstable_by(|a, b| a.key.cmp(&b.key));

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);
    let count = lines.len();
    for i in 0..count {
        let s1 = &lines[i];
        if i > 0 {
            let s2 = &lines[i - 1];
            // 「読み+表記」が重複するエントリはスキップ
            if s1.yomi == s2.yomi && s1.hyouki == s2.hyouki {
                continue;
            }
        }

        let v = format!("{}\t{}\t{}\t{}\t{}\n", &s1.yomi, id, id, &s1.cost, &s1.hyouki);
        writer.write(v.as_bytes())?;
    }

    Ok(())
}

pub fn run_convert_alt_cannadic_to_mozcdic() -> std::io::Result<()> {
    const DATE: &str = "110208";
    const CANNA_FILE1: &str = "gcanna.ctd";
    const CANNA_FILE2: &str = "g_fname.ctd";

    let name = format!("alt-cannadic-{}", DATE);
    let tar_name = format!("{}.tar.bz2", &name);
    let addr = format!("https://ja.osdn.net/dl/alt-cannadic/{}", &tar_name);
    let path_file1 = format!("{}/{}", &name, CANNA_FILE1);
    let path_file2 = format!("{}/{}", &name, CANNA_FILE2);

    command_wait("wget", vec!["-nc", &addr])?;
    command_wait("rm", vec!["-rf", &name])?;
    command_wait("tar", vec!["xf", &tar_name])?;
    command_wait("mv", vec![&path_file1, "."])?;
    command_wait("mv", vec![&path_file2, "."])?;

    convert_alt_cannadic_to_mozcdic(CANNA_FILE1, "mozcdic-ut-alt-cannadic.txt")?;
    convert_alt_cannadic_to_mozcdic(CANNA_FILE2, "mozcdic-ut-alt-cannadic-jinmei.txt")?;

    command_wait("rm", vec!["-rf", &name])?;
    command_wait("rm", vec!["-f", CANNA_FILE1])?;
    command_wait("rm", vec!["-f", CANNA_FILE2])?;

    Ok(())
}
