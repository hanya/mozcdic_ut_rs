
// Ported from convert_sudachidict_to_mozcdic.rb file.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


#[derive(Clone, Debug)]
struct Entry {
    key: String,
    yomi: String,
    hyouki: String,
    cost: String,
}

fn convert_sudachidict_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // sudachidict のエントリから読みと表記を取得
    
    let mut l2 = Vec::with_capacity(1024 * 1024);

    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    while let Ok(1..) = reader.read_line(&mut line) {
        // https://github.com/WorksApplications/Sudachi/blob/develop/docs/user_dict.md
        // 見出し (TRIE 用),左連接ID,右連接ID,コスト,見出し (解析結果表示用),\
        // 品詞1,品詞2,品詞3,品詞4,品詞 (活用型),品詞 (活用形),\
        // 読み,正規化表記,辞書形ID,分割タイプ,A単位分割情報,B単位分割情報,※未使用

        // little glee monster,4785,4785,5000,Little Glee Monster,名詞,固有名詞,一般,*,*,*,\
        // リトルグリーモンスター,Little Glee Monster,*,A,*,*,*,*
        // モーニング娘,5144,5142,10320,モーニング娘,名詞,固有名詞,一般,*,*,*,\
        // モーニングムスメ,モーニング娘。,*,C,*,*,*,*
        // 新型コロナウィルス,5145,5144,13856,新型コロナウィルス,名詞,普通名詞,一般,*,*,*,\
        // シンガタコロナウィルス,新型コロナウイルス,*,C,*,*,*,*
        // アイアンマイケル,5144,4788,9652,アイアンマイケル,名詞,固有名詞,人名,一般,*,*,\
        // アイアンマイケル,アイアン・マイケル,*,C,*,*,*,*

        // midashi, cost, hyouki, kind1, kind3, kind4, yomi
        // 0,       3,    4,      5,     7,     8,     11
        let mut ss = line.split(',');
        let mut midashi = String::from(ss.next().unwrap()); // 0
        ss.next(); // 1
        ss.next(); // 2
        let cost = String::from(ss.next().unwrap()); // 3
        // 「見出し (解析結果表示用)」を表記にする
        let hyouki = String::from(ss.next().unwrap()); // 4
        let kind1 = ss.next().unwrap(); // 5
        ss.next(); // 6
        let kind3 = ss.next().unwrap(); // 7
        let kind4 = ss.next().unwrap(); // 8
        ss.next(); // 9
        ss.next(); // 10
        // 「読み」を取得
        let mut yomi = String::from(ss.next().unwrap()); // 11
        if yomi.find('＝').is_some() || yomi.find('・').is_some() {
            yomi = yomi.replace('＝', "").replace('・', "");
        }

        // 読みのカタカナをひらがなに変換
        // 「tr('ァ-ヴ', 'ぁ-ゔ')」よりnkfのほうが速い
        yomi = to_hiragana_replace_ie(&yomi);

        // 読みがひらがな以外を含む場合はスキップ
        if yomi.chars()
               .any(|c| !(('ぁ' <= c && c <= 'ゔ') || c == 'ー')) {
            line.clear();
            continue;
        }

        // 表記が英数字のみで、表記と「見出し (TRIE 用)」の downcase が同じ場合は表記に揃える
        if hyouki.is_ascii() &&
           hyouki.to_ascii_lowercase() == midashi.to_ascii_lowercase() {
            midashi.clear();
            midashi.push_str(&hyouki);
        }

        // 表記が「見出し (TRIE 用)」と異なる場合はスキップ
        if hyouki != midashi ||
           // 名詞以外の場合はスキップ
           kind1 != "名詞" ||
           // 「地名」をスキップ。地名は郵便番号ファイルから生成する
           kind3 == "地名" ||
           // 「名」をスキップ
           kind4 == "名" {
            line.clear();
            continue;
        }

        // [読み, 表記, コスト] の順に並べる
        l2.push(Entry {
            key: format!("{}\t{}\t{}", yomi, hyouki, cost),
            yomi,
            hyouki,
            cost,
        });
        line.clear();
    }

    let mut lines = l2;
    lines.par_sort_unstable_by(|a, b| a.key.cmp(&b.key));

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
            let mut cost = i32::from_str_radix(&s1.cost, 10).unwrap();

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
        let t = format!("{}\t{}\t{}\t{}\t{}\n", &s1.yomi, id, id, cost, &s1.hyouki);
        writer.write(t.as_bytes())?;
    }

    Ok(())
}

pub fn run_convert_sudachidict_to_mozcdic() -> std::io::Result<()> {
    const TARGET: &str = "/WorksApplications/SudachiDict/commit/";

    // sudachidict ページからコミット情報を取得
    let dictver = {
        command_wait("wget", vec!["https://github.com/WorksApplications/SudachiDict/commits/develop/src/main/text/core_lex.csv", "-O", "sudachidict.html"])?;
        let s = read_file("sudachidict.html")?;
        if let Some((_prefix, suffix)) = s.split_once(TARGET) {
            String::from(suffix.get(..7).unwrap())
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "sudachidic version not found"));
        }
    };
    command_wait("rm", vec!["-f", "sudachidict.html"])?;

    // ファイルをダウンロード
    let corelex = format!("core_lex.{}.csv", dictver);
    let notcorelex = format!("notcore_lex.{}.csv", dictver);

    if !File::open(&corelex).is_ok() {
        remove_matched(".", r"core_lex\..*")?;
        command_wait("wget", vec!["https://github.com/WorksApplications/SudachiDict/raw/develop/src/main/text/core_lex.csv", "-O", &corelex])?;
    } else {
        println!("{} already exists.", &corelex);
    }

    if !File::open(&notcorelex).is_ok() {
        remove_matched(".", r"notcore_lex\..*")?;
        command_wait("wget", vec!["-nc", "https://github.com/WorksApplications/SudachiDict/raw/develop/src/main/text/notcore_lex.csv", "-O", &notcorelex])?;
    } else {
        println!("{} already exists.", &notcorelex);
    }

    convert_sudachidict_to_mozcdic(&corelex, "mozcdic-ut-sudachidict-core.txt")?;
    convert_sudachidict_to_mozcdic(&notcorelex, "mozcdic-ut-sudachidict-notcore.txt")?;

    Ok(())
}
