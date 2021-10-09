
// Ported from generate_jawiki_ut.rb and convert_jawiki_ut_to_mozcdic.rb files.
// Content of convert_jawiki_ut_to_mozcdic.rb is merged into some functions.

use std::io::{BufWriter, Read, Write};
use std::fs::File;
use std::sync::{Arc, Mutex};

use regex::*;
use rayon::{prelude::*, ThreadPoolBuilder};
use bzip2_rs::{decoder::ParallelDecoderReader, RayonThreadPool};
//use bzip2::read::MultiBzDecoder;

use super::mozc::get_id;
use super::util::*;


fn check_jawiki_ut_version() -> std::io::Result<(String, bool)> {
    const INDEX_FILE_NAME: &str = "jawiki-index.html";
    command_wait("wget", vec!["https://dumps.wikimedia.org/jawiki/latest/", "-O", INDEX_FILE_NAME])?;

    let jawiki_index = read_file(INDEX_FILE_NAME)?;
    if let Some((_, date)) = jawiki_index.split_once("jawiki-latest-pages-articles-multistream.xml.bz2</a>") {
        if let Some((date, _)) = date.trim_start().split_once(" ") {
            let utdic = format!("jawiki-ut-{}.txt", date);
            if File::open(&utdic).is_ok() {
                println!("{} already exists.", date);
                return Ok((utdic, true));
            } else {
                remove_matched(".", r"jawiki-ut-.*\.txt")?;
                return Ok((utdic, false));
            }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "date is strange"));
        }
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "index data is broken"));
    }
}


fn generate_jawiki_ut(article: &str, ids: &str, out: Arc<Mutex<Vec<String>>>, re_remove_chars: Regex, re_ref: Regex, re_chars: Regex) {
    // タイトルから表記を作る

    // タイトルを取得
    let title = if let Some((title, _)) = article.split_once("</title>") {
        if let Some((_, title)) = title.split_once("<title>") {
            title
        } else {
            return;
        }
    } else {
        return;
    };

    // 記事を取得
    let article = if let Some((_, article)) = article.split_once("xml:space=\"preserve\">") {
        article
    } else {
        return;
    };

    // タイトルの全角英数を半角に変換してUTF-8で出力
    // -m0 MIME の解読を一切しない
    // -Z1 全角空白を ASCII の空白に変換
    // -W 入力に UTF-8 を仮定する
    // -w UTF-8 を出力する(BOMなし)
    let mut hyouki = ascii_to_halfwidth(title).unwrap_or_else(|| String::from(title));

    // 表記を「 (」で切る
    // 田中瞳 (アナウンサー)
    if let Some((prefix, _)) = hyouki.split_once(" (") {
        hyouki = String::from(prefix);
    }

    // 26文字以上の場合はスキップ。候補ウィンドウが大きくなりすぎる
    if hyouki.chars().count() >= 26 ||
        // 内部用のページをスキップ
        hyouki.find("(曖昧さ回避)").is_some() ||
        hyouki.starts_with("Wikipedia:") ||
        hyouki.starts_with("ファイル:") ||
        hyouki.starts_with("Portal:") ||
        hyouki.starts_with("Help:") ||
        hyouki.starts_with("Template:") ||
        hyouki.starts_with("Category:") ||
        hyouki.starts_with("プロジェクト:") ||
        // スペースがある場合はスキップ
        // 記事のスペースを削除してから「表記(読み」を検索するので、残してもマッチしない。
        hyouki.find(" ").is_some() ||
        // 「、」がある場合はスキップ
        // 記事の「、」で読みを切るので、残してもマッチしない。
        hyouki.find("、").is_some() {
        return;
    }

    // 読みにならない文字を削除したhyouki2を作る
    let hyouki2 = re_remove_chars.replace_all(&hyouki, |_: &Captures| String::new());

    // hyouki2が1文字の場合はスキップ
    if hyouki2.chars().count() <= 1 {
        return;
    }

    // hyouki2がひらがなとカタカナだけの場合は、読みをhyouki2から作る
    // さいたまスーパーアリーナ
    if !hyouki2.chars()
              .any(|c| !(('ぁ' <= c && c <= 'ゔ') || ('ァ' <= c && c <= 'ヴ') || c == 'ー')) {
        let yomi = to_hiragana_replace_ie(&hyouki2);

        let v = format!("{}{}{}\n", &yomi, ids, &hyouki);
        out.lock().unwrap().push(v);
        return;
    }

    // 記事の量を減らす
    let lines = if article.starts_with("{{") {
        // 冒頭の連続したテンプレートを1つにまとめる
        let lines = article.replace("}}\n{{", "");
        // 冒頭のテンプレートを削除
        if let Some(index) = lines.find("}}") {
            String::from(lines.get(index..).unwrap())
        } else {
            lines
        }
    } else {
        String::from(article)
    };

    // 記事を最大200行にする
    // 全部チェックすると時間がかかる。
    let mut n = 0;

    // 記事から読みを作る

    for line in lines.split("\n") {
        n += 1;
        if n == 200 {
            break;
        }

        // 全角英数を半角に変換してUTF-8で出力
        let mut s = if let Some(s) = ascii_to_halfwidth(line) {
            s
        } else {
            String::from(line)
        };

        // 「<ref 」から「</ref>」までを削除
        // '''皆藤 愛子'''<ref>一部のプロフィールが</ref>(かいとう あいこ、[[1984年]]
        // '''大倉 忠義'''（おおくら ただよし<ref name="oricon"></ref>、[[1985年]]
        if s.find("&lt;ref").is_some() {
            s = re_ref.replace(&s, |_: &Captures| String::new()).to_string();
        }

        // スペースと '"「」『』 を削除
        // '''皆藤 愛子'''(かいとう あいこ、[[1984年]]
        s = re_chars.replace_all(&s, |_: &Captures| String::new()).to_string();

        // 「表記(読み」を検索
        let mut yomi = if let Some(index) = s.find(&format!("{}(", &hyouki)) {
            s.get(index + hyouki.len() + 1..).unwrap()
        } else {
            continue;
        };

        // 読みを「)」で切る
        if let Some((pre, _)) = yomi.split_once(")") {
            yomi = pre;
        }
        if yomi.len() == 0 {
            continue;
        }

        // 読みを「[[」で切る
        // ないとうときひろ[[1963年]]
        if let Some((pre, _)) = yomi.split_once("[[") {
            yomi = pre;
        }
        if yomi.len() == 0 {
            continue;
        }

        // 読みを「、」で切る
        // かいとうあいこ、[[1984年]]
        if let Some((pre, _)) = yomi.split_once("、") {
            yomi = pre;
        }
        if yomi.len() == 0 {
            continue;
        }

        // HTMLの特殊文字を変換
        hyouki = hyouki.replace("&amp;", "&");
        hyouki = hyouki.replace("&quot;", "\"");

        // 読みが「ー」で始まる場合はスキップ
        if let Some('ー') = yomi.chars().next() {
            continue;
        }

        // 読みが全てカタカナの場合はスキップ
        // ミュージシャン一覧(グループ)
        if !yomi.chars()
               .any(|c| !(('ァ' <= c && c <= 'ヴ') || c == 'ー')) {
            continue;
        }

        // 読みのカタカナをひらがなに変換
        let yomi = to_hiragana_replace_ie(yomi);

        // 読みがひらがな以外を含む場合はスキップ
        if yomi.chars()
               .any(|c| !(('ぁ' <= c && c <= 'ゔ') || c == 'ー')) {
            continue;
        }

        let v = format!("{}{}{}\n", &yomi, ids, &hyouki);
        out.lock().unwrap().push(v);
        return;
    }
}

const LATEST_FILE_NAME: &str = "jawiki-latest-pages-articles-multistream.xml.bz2";

fn run_thread_generate_jawiki_ut(utdic: &str, dicname: &str) -> std::io::Result<()> {
    let re_remove_chars = Regex::new(r"[!?=:・。]").unwrap();
    let re_ref = Regex::new(r"&lt;ref.*?&lt;/ref&gt;").unwrap();
    let re_chars = Regex::new(r##"[ '"「」『』]"##).unwrap();

    // Mozcの品詞IDを取得
    //「名詞,固有名詞,人名,一般,*,*」は優先度が低いので使わない。
    //「名詞,固有名詞,一般,*,*,*」は後でフィルタリングする。
    let id = get_id(r"(\d*) 名詞,固有名詞,一般,\*,\*,\*,\*")?;
    let ids = format!("\t{}\t{}\t6000\t", id, id);

    // Parallel のプロセス数を (物理コア数) にする
    let core_num = get_core_count()?;
    let pool = ThreadPoolBuilder::new().num_threads(core_num).build().unwrap();

    let fr = File::open(LATEST_FILE_NAME)?;
    // TODO, ParallelDecoderReader is twice faster but makes strange result.
    let mut reader = ParallelDecoderReader::new(fr, RayonThreadPool, 1024 * 1024 * 16);
    //let mut reader = MultiBzDecoder::new(fr);

    const BLOCK_SIZE: usize = 900 * 1000;
    const BUF_SIZE: usize = BLOCK_SIZE * 260;
    // bzip2 can load 900K bytes per read from multistreams archive.

    let read_out = BUF_SIZE;

    let mut broken_bytes = Vec::with_capacity(16);
    let mut remained = Vec::with_capacity(64 * 1024);
    let mut buf = String::with_capacity(BUF_SIZE);

    // We need over 1060000 entries.
    let out = Arc::new(Mutex::new(Vec::with_capacity(1 * 1024 * 1024 + 16 * 1024)));

    loop {
        //println!("Reading...");
        let len = {
            // Copy remained data to start of the buffer if exists.
            let remained_len = remained.len();
            if remained_len > 0 {
                unsafe { buf.as_mut_vec()[0..remained_len].copy_from_slice(&remained) };
                remained.clear();
            }

            // Read data.
            let len = {
                let v = &mut unsafe { buf.as_mut_vec() };
                unsafe { v.set_len(BUF_SIZE) };
                let mut total_len = 0;
                //let it = std::time::Instant::now();
                while total_len + remained_len <= read_out {
                    let l = reader.read(&mut v.as_mut_slice()[total_len + remained_len..]).unwrap();
                    total_len += l;
                    if l == 0 {
                        break;
                    }
                }
                total_len
            };

            // Cut at char boundary to make string UTF-8 safe.
            if len > 0 {
                let index = (0..len + remained_len)
                    .rev()
                    .find(|&i| buf.is_char_boundary(i))
                    .unwrap_or(BUF_SIZE);
                if index != BUF_SIZE && !buf.as_bytes()[index].is_ascii() {
                    broken_bytes.extend_from_slice(&buf.as_bytes()[index..len + remained_len]);
                    unsafe { buf.as_mut_vec().truncate(index) };
                } else {
                    unsafe { buf.as_mut_vec().truncate(len + remained_len) };
                }
            } else {
                unsafe { buf.as_mut_vec().truncate(remained_len) };
            }
            len
        };

        //println!("Writing...");

        pool.scope(|scope| {
            let mut it = buf.split("  </page>").peekable();
            while let Some(s) = it.next() {
                if it.peek().is_some() || len == 0 {
                    let article = s.clone();
                    let ids_ = ids.as_str().clone();
                    let out_ = Arc::clone(&out);
                    let re_remove_chars_ = re_remove_chars.clone();
                    let re_ref_ = re_ref.clone();
                    let re_chars_ = re_chars.clone();
                    scope.spawn(move |_| {
                        generate_jawiki_ut(article, ids_, out_,
                            re_remove_chars_, re_ref_, re_chars_);
                    });
                } else if len != 0 {
                    // 途中で切れた記事をキープ
                    remained.extend_from_slice(s.as_bytes());
                    if !broken_bytes.is_empty() {
                        remained.extend_from_slice(&broken_bytes);
                        broken_bytes.clear();
                    }
                    break;
                }
            }
        });
        if len == 0 {
            break;
        }
    }

    // 重複行を削除
    if let Ok(ref mut mutex) = out.lock() {
        mutex.par_sort_unstable();
        mutex.dedup();

        let d = File::create(dicname)?;
        let mut writer = BufWriter::new(d);
        for line in mutex.iter() {
            writer.write(line.as_bytes())?;
        }
    }

    // create flag, zero contents
    File::create(utdic)?;

    return Ok(());
}

pub fn run_generate_jawiki_ut() -> std::io::Result<()> {
    let dicname = "mozcdic-ut-jawiki.txt";
    let (utdic, state) = check_jawiki_ut_version()?;
    if state {
        return Ok(());
    } else {
        let addr = format!("https://dumps.wikimedia.org/jawiki/latest/{}", LATEST_FILE_NAME);
        command_wait("wget", vec!["-N", "-q", &addr])?;

        run_thread_generate_jawiki_ut(&utdic, dicname)?;
    }

    Ok(())
}
