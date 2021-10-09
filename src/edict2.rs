
// Ported from convert_edict2_to_mozcdic.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use encoding_rs::*;
use regex::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


fn convert_edict2_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Mozcの品詞IDを取得
    // 「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    // 「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,一般,\*,\*,\*,\*,\*")?;
    let re = Regex::new(r"[ ・=]").unwrap();

    let buf = read_file_vec(filename)?;
    let mut l2 = Vec::new();
    let (cow, _encoding_used, _had_errors) = EUC_JP.decode(&buf);
    for line in cow.lines() {
        // 全角スペースで始まるエントリはスキップ
        if line.starts_with("　") {
            continue;
        }

        // 名詞のみを収録
        if let Some((s, _)) = line.split_once(" /(n") {
            // 表記と読みに分ける。表記または読みが複数あるときはそれぞれ最初のものを採用する
            // 脇見(P);わき見;傍視 [わきみ(P);ぼうし(傍視)] /
            let (hyouki, yomi) = if let Some((prefix, suffix)) = s.split_once(" [") {
                let hyouki = prefix.split(";").next().unwrap();
                let yomi = suffix.split(";").next().unwrap();
                (String::from(hyouki), yomi.replace("]", ""))
            } else {
                // カタカナ語には読みがないので表記から読みを作る
                // ブラスバンド(P);ブラス・バンド /(n) brass band/
                let hyouki = s.split(";").next().unwrap();
                (String::from(hyouki), String::from(hyouki))
            };

            let hyouki = hyouki.split("(").next().unwrap();
            let mut yomi = String::from(yomi.split("(").next().unwrap());
            match re.replace_all(&yomi, |_caps: &Captures| {
                String::new() // replace with empty string
            }) {
                std::borrow::Cow::Owned(r) => yomi = r,
                _ => {},
            };

            // 読みのカタカナをひらがなに変換
            yomi = to_hiragana_replace_ie(&yomi);

            l2.push(format!("{}\t{}\t{}\t6000\t{}\n", &yomi, id, id, hyouki));
        }
    }

    let mut lines = l2;

    // 重複行を削除
    lines.par_sort_unstable();
    lines.dedup();

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);
    for line in lines {
        writer.write(line.as_bytes())?;
    }

    Ok(())
}

pub fn run_convert_edict2_to_mozcdic() -> std::io::Result<()> {
    const FILE_NAME: &str = "edict2";
    const DIC_NAME: &str = "mozcdic-ut-edict2.txt";

    let gz_name = format!("{}.gz", FILE_NAME);
    let addr = format!("http://ftp.edrdg.org/pub/Nihongo/{}", &gz_name);

    command_wait("rm", vec!["-f", FILE_NAME])?;
    command_wait("wget", vec!["-N", "-q", &addr])?;
    command_wait("gzip", vec!["-dk", &gz_name])?;

    convert_edict2_to_mozcdic(FILE_NAME, DIC_NAME)?;

    Ok(())
}
