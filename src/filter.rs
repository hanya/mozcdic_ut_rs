
// Ported from filter_unsuitable_entries.rb and remove_unnecessary_entries.rb files.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use regex::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


fn to_halfwidth_ascii(s: &str) -> String {
    s.chars().map(|c| match c {
        '！'..='｝' => char::from_u32(c as u32 - '！' as u32 + '!' as u32).unwrap(),
        _ => c,
    }).collect()
}

fn collect_numbers(v: &str) -> Option<i32> {
    let mut s = String::new();
    v.chars().for_each(|c| if '0' <= c && c <= '9' { s.push(c); });
    if s.len() > 0 { i32::from_str_radix(&s, 10).ok() } else { None }
}


pub fn remove_unnecessary_entries(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Remove some hangul here too.
    let re = Regex::new("[ !?=:・。★☆\u{1100}-\u{11FF}\u{A960}-\u{A97F}\u{D7B0}-\u{D7FF}]").unwrap();

    let mut l2 = Vec::new();

    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    while let Ok(1..) = reader.read_line(&mut line) {
        let mut ss = line.split("\t");
        if let (Some(yomi), Some(id), Some(_), Some(cost), Some(hyouki)) =
            (ss.next(), ss.next(), ss.next(), ss.next(), ss.next()) {
            let mut hyouki = String::from(hyouki.trim_end());
            let mut yomi = String::from(yomi);

            // 表記の全角英数を半角に変換
            hyouki = to_halfwidth_ascii(&hyouki);

            // 表記の「~」を「〜」に置き換える
            // jawiki-latest-all-titles の表記に合わせる。
            hyouki = hyouki.replace("~", "〜");

            // 表記の最初が空白の場合は取る
            if hyouki.starts_with(" ") {
                hyouki = String::from(hyouki.trim_start());
            }

            // 表記の全角カンマを半角に変換
            hyouki = hyouki.replace("，", ", ");

            // 表記の最後が空白の場合は取る（「, 」もここで処理）
            if hyouki.ends_with(" ") {
                hyouki = String::from(hyouki.trim_end());
            }

            // 読みにならない文字を削除したhyouki2を作る
            let hyouki2 = re.replace(&hyouki, "").to_owned();

            // hyouki2がひらがなとカタカナだけの場合は、読みをhyouki2から作る
            // さいたまスーパーアリーナ
            if yomi.chars()
               .any(|c| !(('ぁ' <= c && c <= 'ゔ') || ('ァ' <= c && c <= 'ヴ') || c == 'ー')) {
                yomi = to_hiragana_replace_ie(&hyouki2);
            }

            //let yomi_len = yomi.chars().count();
            let (yomi_len, yomi_hira_len) = yomi.chars().fold((0, 0), |(count, hira_count), c| {
                if 'ぁ' <= c && c <= 'ゔ' { (count + 1, hira_count + 1) } else { (count + 1, hira_count) }
            });
            let hyouki2_len = hyouki2.chars().count();

            // 読みが2文字以下の場合はスキップ
            if yomi_len <= 2 ||
                // hyouki2が1文字の場合はスキップ
                hyouki2_len <= 1 ||
                // hyoukiが26文字以上の場合はスキップ
                hyouki.chars().count() >= 26 ||
                // 読みの文字数がhyouki2の4倍を超える場合はスキップ
                // けやきざかふぉーてぃーしっくす（15文字） 欅坂46（4文字）
                yomi_len > hyouki2_len * 4 ||
                // hyouki2の文字数が読みの文字数より多い場合はスキップ
                // 英数字表記が削除されるのを防ぐため、hyouki2の文字数は (bytesize / 3) とする。
                // みすたーちるどれんりふれくしょん（16文字） Mr.Children REFLECTION（22bytes / 3）
                // あいしす（16文字） アイシス（48bytes / 3）
                yomi_len < hyouki2.len() / 3 ||
                // 読みがひらがな以外を含む場合はスキップ
                yomi_len != yomi_hira_len ||
                // hyoukiがコードポイントを含む場合はスキップ
                // デコードする場合
                // hyouki = hyouki.gsub(/\\u([\da-fA-F]{4})/){[$1.hex].pack("U")}
                hyouki.find("\\u").is_some() ||
                // hyouki2の数字が101以上の場合はスキップ（100円ショップを残す）
                // 国道120号, 3月26日
                collect_numbers(&hyouki2).unwrap_or(0) > 100 {
                line.clear();
                continue;
            }

            //l2.push(format!("{}\t{}\t{}\t{}\t{}\n", &yomi, id, id, cost, &hyouki));
            l2.push(format!("{}\t{}\t{}\t{}\t{}\n", &yomi, &hyouki, cost, id, id));
        }
        line.clear();
    }

    let mut lines = l2;

    // UT辞書の並びを変える
    // (変更前) げんかん	1823	1823	5278	玄関
    // (変更後) げんかん	玄関	5278	1823	1823
    // done by chainging order in the function above

    // Mozc辞書の並びを変えてマークをつける
    // (変更前) げんかん	1823	1823	6278	玄関
    // (変更後) げんかん	玄関	*6278	1823	1823
    {
        let mut line = String::new();
        let f = File::open("mozcdic.txt")?;
        let mut reader = BufReader::new(f);
        while let Ok(1..) = reader.read_line(&mut line) {
            let mut ss = line.trim_end().split('\t');
            if let (Some(yomi), Some(id), Some(_), Some(cost), Some(hyouki)) =
                (ss.next(), ss.next(), ss.next(), ss.next(), ss.next()) {
                lines.push(format!("{}\t{}\t*{}\t{}\t{}\n", yomi, hyouki, cost, id, id));
            }
        }
    }

    lines.par_sort_unstable();

    // この時点での並び。Mozc辞書が先になる
    // げんかん	玄関	*6278	1823	1823
    // げんかん	玄関	5278	1823	1823

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);

    let count = lines.len();
    for i in 0..count {
        let s1 = &lines[i];
        let mut ss1 = s1.split('\t');
        let (s1_yomi, s1_hyouki, s1_cost) =
            (ss1.next().unwrap(), ss1.next().unwrap(), ss1.next().unwrap());

        // Mozc辞書はスキップ
        if s1_cost.starts_with('*') {
            continue;
        }

        if i > 0 {
            let s2 = &lines[i - 1];
            let mut ss2 = s2.split('\t');
            let (s2_yomi, s2_hyouki, s2_cost) =
                (ss2.next().unwrap(), ss2.next().unwrap(), ss2.next().unwrap());

            // Mozc辞書と「読み+表記」が重複するUT辞書はスキップ
            if s2_cost.starts_with('*') &&
                (s1_yomi == s2_yomi && s1_hyouki == s2_hyouki) {
                continue;
            }

            // UT辞書内で重複するエントリをコスト順にスキップ
            if !s2_cost.starts_with('*') &&
                (s1_yomi == s2_yomi && s1_hyouki == s2_hyouki) {
                continue;
            }
        }

        let id = ss1.next().unwrap();

        let v = format!("{}\t{}\t{}\t{}\t{}\n", s1_yomi, id, id, s1_cost, s1_hyouki);
        writer.write(v.as_bytes())?;
    }

    Ok(())
}

pub fn filter_unsuitable_entries(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Mozc形式に変換した辞書を読み込む
    // なかいまさひろ	1917	1917	6477	中居正広
    let lines = read_file(filename)?;

    // フィルタリング対象のIDを取得
    // 品詞IDを取得
    let id = get_id(r"(\d*) 名詞,固有名詞,一般,\*,\*,\*,\*")?;

    // 単語フィルタを読み込む
    let filter_data = read_file("../data/filter/unsuitable-entries.txt")?;

    // エントリが正規表現になっているときは正規表現を作る
    // /\Aバカ/
    let mut exp = Vec::new();
    for line in filter_data.lines() {
        let s = line.trim_end();
        if s.starts_with("/") && s.ends_with("/") {
            if s.ends_with("\\Z/") {
                let mut v = String::from(unsafe { s.get_unchecked(1..s.len() - 3) });
                v.push_str("\\z");
                exp.push(v);
            } else {
                exp.push(String::from(unsafe { s.get_unchecked(1..s.len() - 1) }));
            }
        } else {
            exp.push(String::from(s));
        }
    }
    let res = RegexSet::new(&exp).unwrap();

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);

    for line in lines.lines() {
        let mut s = line.split("\t");
        if let (Some(_reading), Some(tid), Some(_), Some(_cost), Some(writing)) =
            (s.next(), s.next(), s.next(), s.next(), s.next()) {

            // フィルタリング対象のIDの場合は実行
            if tid == &id && res.is_match(writing) {
                continue;
            }

            writer.write(line.as_bytes())?;
            writer.write(b"\n")?;
        }
    }

    Ok(())
}
