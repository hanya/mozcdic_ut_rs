
// Ported from fix_ken_all.rb and generate_chimei_for_mozcdic.rb files.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use encoding_rs::*;
use regex::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


pub fn generate_chimei_for_mozcdic() -> std::io::Result<()> {
    // Mozcの品詞IDを取得
    let id = get_id(r"(\d*) 名詞,固有名詞,地域,一般,\*,\*,\*")?;

    let re_num = Regex::new(r"\d+").unwrap();
    let number_to_reading = {
        // 半角数字をひらがなに変換する配列を作成
        let d1 = ["", "いち", "に", "さん", "よん", "ご", "ろく", "なな", "はち", "きゅう"];

        // d1[10] から d1[59] までのひらがなを作成
        // さっぽろしひがしくきた51じょうひがし
        let d2 = ["", "じゅう", "にじゅう", "さんじゅう", "よんじゅう", "ごじゅう"];

        let mut dd = Vec::with_capacity(64);
        for dp in d2 {
            for d in d1 {
                dd.push(format!("{}{}", dp, d));
            }
        }
        dd
    };

    let mut l2 = Vec::new();
    let d = File::open("KEN_ALL.CSV.fixed")?;
    let mut reader = BufReader::new(d);
    let mut line = String::new();
    while let Ok(1..) = reader.read_line(&mut line) {
        let mut s = line.replace("\"", "")
                        .split(",")
                        .map(String::from)
                        .collect::<Vec<String>>();

        // 並びの例
        // "トヤマケン","タカオカシ","ミハラマチ","富山県","高岡市","美原町"
        // s[3], s[4], s[5], s[6], s[7], s[8]

        // 読みをひらがなに変換
        s[3] = half_to_hiragana_no_dot(&s[3]);
        s[4] = half_to_hiragana_no_dot(&s[4]);
        s[5] = half_to_hiragana_no_dot(&s[5]);

        // 読みの「・」を取る
        // removed in half_to_hiragana_no_dot

        // 市を出力
        let l = format!("{}\t{}\t{}\t9000\t{}\n", &s[4], id, id, &s[7]);
        l2.push(l);

        // 町の読みが半角数字を含むか確認
        // 町の読みの半角数字が59以下の場合はひらがなに変換
        // さっぽろしひがしくきた51じょうひがし
        match re_num.replace_all(&s[5], |caps: &Captures| {
            let index = usize::from_str_radix(caps.get(0).unwrap().as_str(), 10).unwrap();
            String::from(if index < 60 {
                &number_to_reading[index]
            } else {
                caps.get(0).unwrap().as_str()
            })
        }) {
            std::borrow::Cow::Owned(r) => s[5] = r,
            _ => {},
        }

        // 町の読みがひらがな以外を含む場合はスキップ
        // 「自由が丘(3～7丁目)」「OAPたわー」
        if s[5].chars()
               .any(|c| !(('ぁ' <= c && c <= 'ゔ') || c == 'ー')) ||
           // 町の表記が空の場合はスキップ
           s[8] == "" {
            line.clear();
            continue;
        }

        // 町を出力
        l2.push(format!("{}\t{}\t{}\t9000\t{}\n", &s[5], id, id, &s[8]));

        // 市+町を出力
        l2.push(format!("{}{}\t{}\t{}\t9000\t{}{}\n", &s[4], &s[5], id, id, &s[7], &s[8]));

        line.clear();
    }

    // 重複行を削除
    l2.par_sort_unstable();
    l2.dedup();

    let f = File::create("mozcdic-ut-chimei.txt")?;
    let mut writer = BufWriter::new(f);
    for line in l2 {
        writer.write(line.as_bytes())?;
    }

    Ok(())
}

fn fix_ken_all(filename: &str, dicname: &str) -> std::io::Result<()> {
    let d = File::create(dicname)?;
    let mut writer = BufWriter::new(d);

    let re = Regex::new(r"[０-９ａ-ｚＡ-Ｚ（）　−]").unwrap();
    // 除外する文字列
    // (例) 「3701、3704、」「4710〜4741」「坪毛沢「2」」
    let ngs = ["○", "〔", "〜", "、", "「", "を除く", "以外", "その他",
              "地割", "不明", "以下に掲載がない場合"];

    let buf = read_file_vec(filename)?;
    let (cow, _encoding_used, _had_errors) = SHIFT_JIS.decode(&buf);
    for line in cow.lines() {
        // 並びの例
        // 46201,"89112","8911275","カゴシマケン","カゴシマシ", "カワカミチョウ(3649)",
        // "鹿児島県","鹿児島市","川上町（３６４９）"

        let mut s = line.split(",").map(String::from).collect::<Vec<String>>();
        match re.replace_all(&s[8], |caps: &Captures| {
            let mut rs = String::with_capacity(16);
            for c in caps.get(0).unwrap().as_str().chars() {
                match c {
                    '０'..='９' => rs.push(char::from_u32('0' as u32 + c as u32 - '０' as u32).unwrap()),
                    'ａ'..='ｚ' => rs.push(char::from_u32('a' as u32 + c as u32 - 'ａ' as u32).unwrap()),
                    'Ａ'..='Ｚ' => rs.push(char::from_u32('A' as u32 + c as u32 - 'Ａ' as u32).unwrap()),
                    '（'..='）' => rs.push(char::from_u32('(' as u32 + c as u32 - '（' as u32).unwrap()),
                    '　' => rs.push(' '),
                    '−' => rs.push('-'),
                    _ => {},
                }
            }
            rs
        }) {
            std::borrow::Cow::Owned(r) => s[8] = r,
            _ => {},
        }

        // 町域表記の () 内に除外文字列があるかチェック
        if let Some(index) = s[8].find("(") {
            let t = unsafe { s[8].get_unchecked(index..s[8].len() - 2) };
            for ng in ngs {
                if t.find(ng).is_some() {
                    // 該当する場合は町域の読みと表記の「(」以降を削除
                    if let Some(index5) = s[5].find("(") {
                        s[5] = String::from(unsafe { s[5].get_unchecked(..index5) });
                    }
                    if let Some(index8) = s[8].find("(") {
                        s[8] = String::from(unsafe { s[8].get_unchecked(..index8) });
                    }
                    break;
                }
            }
        }

        // 町域表記の () 外に除外文字列があるかチェック
        for ng in ngs {
            if s[8].find(ng).is_some() {
                // 該当する場合は町域の読みと表記を "" にする
                s[5].clear();
                s[8].clear();
                break;
            }
        }

        // 町域の読みの () を取る
        // (例) 「"ハラ(ゴクラクザカ)","原(極楽坂)"」を
        // 「"ハラゴクラクザカ","原(極楽坂)"」にする。
        // 表記の () はそのままにする。「原極楽坂」だと読みにくいので
        s[5] = s[5].replace("(", "").replace(")", "");

        for (index, ss) in s.iter().enumerate() {
            writer.write(ss.as_bytes())?;
            if index < 14 {
                writer.write(b",")?;
            }
        }
        writer.write(b"\n")?;
    }

    Ok(())
}

pub fn run_fix_ken_all() -> std::io::Result<()> {
    const KEN_NAME: &str = "KEN_ALL.CSV";

    command_wait("rm", vec!["-f", KEN_NAME])?;
    command_wait("wget", vec!["-N", "https://www.post.japanpost.jp/zipcode/dl/kogaki/zip/ken_all.zip"])?;
    command_wait("unzip", vec!["ken_all.zip"])?;
    fix_ken_all("KEN_ALL.CSV", "KEN_ALL.CSV.fixed")?;
    command_wait("rm", vec!["-f", KEN_NAME])?;

    Ok(())
}
