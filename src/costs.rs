
// Ported from calculate_costs.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use rayon::prelude::*;

use super::util::*;


#[derive(Clone)]
struct Entry<'a> {
    key: String,
    original: &'a str,
    reading: &'a str,
    id1: &'a str,
    id2: &'a str,
    cost: &'a str,
    writing: &'a str,
    /// Cost in numeric value.
    cost_num: i32,
    wiki_title: bool,
    remove_flag: bool,
}

pub fn calculate_costs(filename: &str, dicname: &str) -> std::io::Result<()> {
    // jawikiの見出し語ヒット数を読み込む
    // jawikititles	0	0	34	中居正広
    let mut data = read_file("jawiki-latest-all-titles-in-ns0.hits")?;

    // 追加辞書を加える
    // なかいまさひろ	1916	1916	6477	中居正広
    {
        let add = read_file(filename)?;
        if !data.ends_with('\n') {
            data.push('\n');
        }
        data.push_str(&add);
    }

    let mut entries: Vec<Entry> = data.trim_end().split("\n").map(|line| {
        let mut ss = line.split('\t');
        let mut entry = Entry {
            key: String::new(),
            original: line,
            reading: ss.next().unwrap(),
            id1: ss.next().unwrap(),
            id2: ss.next().unwrap(),
            cost: ss.next().unwrap(),
            writing: ss.next().unwrap(),
            cost_num: 0,
            wiki_title: false,
            remove_flag: false,
        };
        entry.key = format!("{}\t{}\t{}\t{}\t{}", entry.writing, entry.reading, entry.id1, entry.id2, entry.cost);
        entry.cost_num = i32::from_str_radix(&entry.cost, 10).unwrap();
        entry.wiki_title = entry.reading == "jawikititles";
        entry
    }).collect();

    // jawikiヒット数の下に追加辞書が来るよう並べ替える
    // 中居正広	jawikititles	0	0	34
    // 中居正広	なかいまさひろ	1847	1847	5900
    // 中居正広	なかいまさひろ	1917	1917	6477
    entries.par_sort_unstable_by(|a, b| a.key.cmp(&b.key));

    let mut jawiki = entries[0].clone();

    for mut entry in entries.iter_mut() {
        // jawikiの見出し語を取得
        // 中居正広	jawikititles	0	0	34

        if entry.wiki_title {
            jawiki = entry.clone();

            // jawikiの見出し語は後でまとめて削除
            entry.remove_flag = true;

            // jawikiのヒット数が大きいときは抑制
            if jawiki.cost_num > 30 {
                jawiki.cost_num = 30;
            }

            continue;
        }

        // jawikiの見出し語にヒットしない英数字のみの表記は除外
        if entry.writing != jawiki.writing {
            if entry.writing.chars().count() == entry.writing.len() {
                entry.remove_flag = true;
                continue;
            }

            // jawikiの見出し語にヒットしない表記はコストのベースを8000にする
            // コスト = 8000 + (元のコスト値/10)
            entry.cost_num = 8000 + entry.cost_num / 10;
            continue;
        }

        // jawikiの見出し語に1回ヒットする表記はコストのベースを7000にする
        // 中居正広	なかいまさひろ	1917	1917	6477
        // コスト値 = 7000 + (元のコスト値/10)
        if jawiki.cost_num == 1 {
            entry.cost_num = 7000 + entry.cost_num / 10;
            continue;
        }

        // jawikiの見出し語に2回以上ヒットする表記はコストのベースを6000にする
        // コスト = 6000 + (元のコスト値/10) - (ヒット数*30)
        entry.cost_num = 6000 + entry.cost_num / 10 - jawiki.cost_num * 30;
    }

    // Mozc形式の並びに戻す
    entries.par_sort_unstable_by(|a, b| a.original.cmp(&b.original));

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);
    for entry in entries {
        if !entry.remove_flag {
            let l = format!("{}\t{}\t{}\t{}\t{}\n",
                entry.reading, entry.id1, entry.id2, entry.cost_num, entry.writing);
            writer.write(l.as_bytes())?;
        }
    }

    Ok(())
}
