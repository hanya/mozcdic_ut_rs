
// Ported from convert_neologd_to_mozcdic.rb file.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use regex::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Entry {
    yomi: String,
    hyouki: String,
    cost: i32,
}

fn convert_neologd_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // mecab-user-dict-seedを読み込む
    // Over 2610000 entries before sorting.
    let mut l2 = Vec::with_capacity(1024 * 1024 * 2 + 1024 * 512);

    let d = File::open(filename)?;
    let mut reader = BufReader::new(d);
    let mut line = String::new();
    // neologd のエントリから読みと表記を取得
    while let Ok(1..) = reader.read_line(&mut line) {
        // 表層形,左文脈ID,右文脈ID,コスト,品詞1,品詞2,品詞3,品詞4,品詞5,品詞6,\
        // 原形,読み,発音
        // little glee monster,1289,1289,2098,名詞,固有名詞,人名,一般,*,*,\
        // Little Glee Monster,リトルグリーモンスター,リトルグリーモンスター
        // リトルグリーモンスター,1288,1288,-1677,名詞,固有名詞,一般,*,*,*,\
        // Little Glee Monster,リトルグリーモンスター,リトルグリーモンスター
        // 新型コロナウィルス,1288,1288,4808,名詞,固有名詞,一般,*,*,*,\
        // 新型コロナウィルス,シンガタコロナウィルス,シンガタコロナウィルス
        // 新型コロナウイルス,1288,1288,4404,名詞,固有名詞,一般,*,*,*,\
        // 新型コロナウイルス,シンガタコロナウイルス,シンガタコロナウイルス

        let mut s = line.split(",");
        // cost, kind1, kind3, kind4, genkei(hyouki), yomi
        // 3,    4,     6,     7,     10,             11
        s.next(); s.next(); s.next(); // 0-2
        let cost = i32::from_str_radix(s.next().unwrap(), 10).unwrap(); // 3
        let kind1 = s.next().unwrap(); // 4
        s.next();
        let kind3 = s.next().unwrap(); // 6
        let kind4 = s.next().unwrap(); // 7
        s.next(); s.next(); // 8-9
        // 「原形」を表記にする
        let hyouki = String::from(s.next().unwrap()); // 10
        // 「読み」を取得
        let mut yomi = String::from(s.next().unwrap()); // 11

        // 読みのカタカナをひらがなに変換
        yomi = to_hiragana_replace_ie(&yomi);

        // 読みがひらがな以外を含む場合はスキップ
        if yomi.chars().any(|c| !(('ぁ' <= c && c <= 'ゔ') || c == 'ー')) {
            line.clear();
            continue;
        }

        // 名詞以外の場合はスキップ
        if kind1 != "名詞" ||
           // 「地域」をスキップ。地名は郵便番号ファイルから生成する
           kind3 == "地域" ||
           // 「名」をスキップ
           kind4 == "名" {
            line.clear();
            continue;
        }

        // [読み, 表記, コスト] の順に並べる
        l2.push(Entry {
            yomi,
            hyouki,
            cost,
        });

        line.clear();
    }

    let mut lines = l2;
    lines.par_sort_unstable();

    // Mozcの品詞IDを取得
    // 「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    // 「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,固有名詞,一般,\*,\*,\*,\*")?;

    // Mozc形式で書き出す
    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);

    let count = lines.len();
    for i in 0..count {
        let s1 = &lines[i];
        if i > 0 {
            let s2 = &lines[i - 1];
            // [読み..表記] が重複する場合はスキップ
            if s1.yomi == s2.yomi && s1.hyouki == s2.hyouki {
                continue;
            }
        }

        let cost = {
            //let mut cost = i32::from_str_radix(&s1.cost, 10).unwrap();
            let mut cost = s1.cost;

            // コストがマイナスの場合は8000にする
            if cost < 0 {
                cost = 8000;
            }

            // コストが10000を超える場合は10000にする
            if cost > 10000 {
                cost = 10000;
            }

            // コストを 6000 < cost < 7000 に調整する
            6000 + (cost / 10)
        };

        // [読み,id,id,コスト,表記] の順に並べる
        let v = format!("{}\t{}\t{}\t{}\t{}\n", &s1.yomi, id, id, cost, &s1.hyouki);
        writer.write(v.as_bytes())?;
    }

    Ok(())
}

pub fn run_convert_neologd_to_mozcdic() -> std::io::Result<()> {
    const URL: &str = "https://github.com/neologd/mecab-ipadic-neologd/tree/master/seed";
    const SEED_NAME: &str = "seed.html";

    let re = Regex::new(r"mecab-user-dict-seed.(\d*).csv.xz").unwrap();

    command_wait("wget", vec![URL, "-O", SEED_NAME])?;

    let neologdver = {
        let s = read_file(SEED_NAME)?;
        if let Some(captures) = re.captures(&s) {
            String::from(captures.get(1).unwrap().as_str())
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "neologd version not found"));
        }
    };

    command_wait("rm", vec![SEED_NAME])?;

    let file_name = format!("mecab-user-dict-seed.{}.csv", neologdver);
    let archive_name = format!("{}.xz", &file_name);
    let addr = format!("https://github.com/neologd/mecab-ipadic-neologd/raw/master/seed/{}", &archive_name);

    if !File::open(&archive_name).is_ok() {
        command_wait("wget", vec!["-nc", &addr])?;
    }
    if !File::open(&file_name).is_ok() {
        command_wait("7z", vec!["x", "-aos", &archive_name])?;
    }

    convert_neologd_to_mozcdic(&file_name, "mozcdic-ut-neologd.txt")?;

    Ok(())
}
