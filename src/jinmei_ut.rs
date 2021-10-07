
// Ported from convert_jinmei_ut_to_mozcdic.rb file.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use rayon::prelude::*;

use super::mozc::get_id;


fn convert_jinmei_ut_to_mozcdic(filename: &str, dicname: &str) -> std::io::Result<()> {
    // Mozcの品詞IDを取得
    // 「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    // 「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,一般,\*,\*,\*,\*,\*")?;

    let mut lines = Vec::new();
    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    while let Ok(1..) = reader.read_line(&mut line) {
        let yomi = line.get(0..line.find("\t").unwrap()).unwrap();
        let hyouki = line.get(line.rfind("\t").unwrap() + 1..).unwrap().trim_end();

        lines.push(format!("{}\t{}\t{}\t6000\t{}\n", yomi, id, id, hyouki));

        line.clear();
    }

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

pub fn run_convert_jinmei_ut_to_mozcdic() -> std::io::Result<()> {
    const FILE_NAME: &str = "../data/jinmei-ut/jinmei-ut.txt";
    const DIC_NAME: &str = "mozcdic-ut-jinmei.txt";

    convert_jinmei_ut_to_mozcdic(FILE_NAME, DIC_NAME)?;

    Ok(())
}
