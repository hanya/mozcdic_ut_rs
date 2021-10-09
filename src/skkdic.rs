
// Ported from convert_skkdic_to_mozcdic.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use encoding_rs::*;
use rayon::prelude::*;

use super::mozc::get_id;
use super::util::*;


fn convert_skkdic_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Mozcの品詞IDを取得
    // 「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    // 「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,一般,\*,\*,\*,\*,\*")?;

    let buf = read_file_vec(filename)?;
    let mut l2 = Vec::new();
    let (cow, _encoding_used, _had_errors) = EUC_JP.decode(&buf);
    for line in cow.lines() {
        // わりふr /割り振/割振/
        // いずみ /泉/和泉;地名,大阪/出水;地名,鹿児島/
        if let Some((yomi, hyoukis)) = line.split_once(" /") {
            let yomi = yomi.replace("う゛", "ゔ");

            // 読みが英数字を含む場合はスキップ
            if yomi.len() != yomi.chars().count() * 3 {
                continue;
            }

            let mut last_normalized = None;
            let mut hyouki: Vec<String> = hyoukis.split_terminator('/').map(String::from).collect();
            let count = hyouki.len();
            for i in 0..count {
                if hyouki[i].len() == 0 {
                    continue;
                }

                if let Some((prefix, _)) = hyouki[i].split_once(';') {
                    hyouki[i] = String::from(prefix);
                }

                // 表記に優先度をつける
                let cost = 7000 + (10 * i);

                // 2個目以降の表記が前のものと重複している場合はスキップ
                // ＩＣカード/ICカード/
                let current_normalized = ascii_to_halfwidth(&hyouki[i]);
                if let Some(last) = &last_normalized {
                    if let Some(current) = &current_normalized {
                        if last == current {
                            continue;
                        }
                    } else {
                        if last == &hyouki[i] {
                            continue;
                        }
                    }
                } else {
                    if i > 0 {
                        if let Some(current) = &current_normalized {
                            if &hyouki[i - 1] == current {
                                continue;
                            }
                        } else {
                            if &hyouki[i - 1] == &hyouki[i] {
                                continue;
                            }
                        }
                    }
                }
                last_normalized = current_normalized;

                l2.push(format!("{}\t{}\t{}\t{}\t{}\n", yomi, id, id, cost, hyouki[i]));
            }
        } else {
            continue;
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

pub fn run_convert_skkdic_to_mozcdic() -> std::io::Result<()> {
    command_wait("wget", vec!["-N", "-q", "http://openlab.jp/skk/dic/SKK-JISYO.L.gz"])?;
    command_wait("rm", vec!["-f", "SKK-JISYO.L"])?;
    command_wait("gzip", vec!["-dk", "SKK-JISYO.L.gz"])?;

    convert_skkdic_to_mozcdic("SKK-JISYO.L", "mozcdic-ut-skkdic.txt")?;

    Ok(())
}
