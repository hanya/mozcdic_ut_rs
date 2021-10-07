
// Ported from add_search_results_to_each_title.rb file.

use std::io::{BufWriter, Write};
use std::fs::File;

use rayon::prelude::*;

use super::util::*;


fn add_search_results_to_each_title(filename: &str, dicname: &str) -> std::io::Result<()> {
    let titles = command_wait_output("gzip", vec!["-c", "-d", filename])?;

    let mut l2 = Vec::new();
    for line in titles.split("\n") {
        // "BEST_(三浦大知のアルバム)" を
        // "三浦大知のアルバム)" に変更。
        // 「三浦大知」を前方一致検索できるようにする
        let mut ss = line.split("_(");

        let data = ss.next().map_or(line, |_| ss.next().map_or(line, |bk| bk));

        // 表記が2文字以下の場合はスキップ
        if data.len() < 3 {
            continue;
        }

        // "_" を " " に置き換える
        // THE_BEATLES
        l2.push(data.replace('_', " "));
    }

    let mut titles = l2;
    titles.par_sort_unstable();

    let f = File::create(dicname)?;
    let mut writer = BufWriter::new(f);

    let len = titles.len();
    for i in 0..len {
        let current = &titles[i];
        // 重複行をスキップ
        // カウント対象として必要なので削除はしない。
        if i > 0 && current == &titles[i - 1] {
            continue;
        }

        let mut count = 1;

        // 前方一致する限りカウントし続ける
        while (i + count) < len && titles[i + count].starts_with(current) {
            count += 1;
        }

        let v = format!("jawikititles\t0\t0\t{}\t{}\n", count, current);
        writer.write(v.as_bytes())?;
    }

    Ok(())
}

pub fn run_add_search_results_to_each_title() -> std::io::Result<()> {
    const FILE_NAME: &str = "jawiki-latest-all-titles-in-ns0.gz";
    const DIC_NAME: &str = "jawiki-latest-all-titles-in-ns0.hits";

    command_wait("wget", vec!["-N", "https://dumps.wikimedia.your.org//jawiki/latest/jawiki-latest-all-titles-in-ns0.gz"])?;

    add_search_results_to_each_title(FILE_NAME, DIC_NAME)?;

    //command_wait("rm", vec!["-f", "jawiki-latest-all-titles-in-ns0"])?;

    Ok(())
}
