#![allow(dead_code)]
#![allow(unused_imports)]

use chrono::{DateTime, FixedOffset, TimeZone, Utc, Date, NaiveDate, LocalResult};
use pulldown_cmark::{Parser, Options, html};
use serde::Serialize;
use std::{fs::{File, self, DirEntry}, io::{Read, Write}, ffi::{OsStr, OsString}, path::{Path, PathBuf, self}, process, collections::HashMap, ops::Index};
use anyhow::{Result, bail};
use handlebars::{Handlebars, Template};
use regex::{Regex, RegexBuilder};

#[derive(Serialize, Default, Debug, Clone)]
struct Article {
    title: String,
    date: String,
    contents: String,
    filename: String,
}

#[derive(Serialize, Debug, Clone)]
struct Context<'a> {
    title: String,
    articles: &'a Vec<Article>,
}

fn load_file(f: impl AsRef<Path>) -> Result<String> {
    let mut f = File::open(f.as_ref())?;
    
    let mut result = String::new();
    f.read_to_string(&mut result)?;

    Ok(result)
}

fn write_file(f: impl AsRef<Path>, contents: impl AsRef<str>) -> Result<()> {
    let mut file = File::create(f.as_ref())?;
    file.write_all(contents.as_ref().as_bytes())?;
    Ok(())
}

fn render_markdown(raw_contents: impl AsRef<str>) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(raw_contents.as_ref(), options);
    
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}

fn get_title(f: impl AsRef<str>) -> Result<String> {
    static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
    let re = RE.get_or_init(|| { Regex::new(r"(?m)^#\s+(.*)$").unwrap() });

    let matches = match re.captures(f.as_ref()) {
        Some(matches) => { matches },
        None => { bail!("Failed to get title"); }
    };

    Ok(matches.get(1).unwrap().as_str().to_string())
}

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    for entry in fs::read_dir(src.as_ref())? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let dst_path = dst.as_ref().join(entry.file_name());
            fs::create_dir_all(&dst_path)?;
            copy_dir(&entry.path(), &dst_path)?;
        } else if entry.file_type()?.is_file() {
            if entry.path().extension().unwrap_or(OsStr::new("")) != "html" {
                if entry.file_name() != OsStr::new(".DS_Store") {
                    let dst_file = dst.as_ref().join(entry.file_name());
                    println!("{} => {}", entry.path().display(), dst_file.display());
                    fs::copy(&entry.path(), &dst_file)?;
                }
            }
        }
    }
    Ok(())
}

fn clean_dir(path: impl AsRef<Path>) -> Result<()> {
    for entry in fs::read_dir(path.as_ref())? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            clean_dir(entry.path())?;
            fs::remove_dir(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let re = Regex::new(r"^((\d{8}).*)\.md$")?;
    let mut articles: Vec<Article> = Vec::new();
    for entry in fs::read_dir("posts")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let filename = entry.file_name().into_string().unwrap();
            let matches = re.captures(&filename);
            if let Some(matches) = matches {
                let date_str = matches.get(2).unwrap().as_str();
                let ndate = NaiveDate::parse_from_str(&date_str, "%Y%m%d")?;
                let date = Date::<Utc>::from_utc(ndate, Utc);

                let contents = load_file(&entry.path())?;
                match get_title(&contents) {
                    Ok(title) => {
                        let contents = render_markdown(&contents)?;
                        let filename = format!("{}.html", matches.get(1).unwrap().as_str());
                        articles.push(Article {
                            title,
                            date: date.format("%Y-%m-%d").to_string(),
                            contents,
                            filename
                        });
                    },
                    Err(_) => {
                        eprintln!("Failed to get title for {}", entry.path().display());
                        process::exit(1);
                    }
                }
            }
        }
    }

    /* Sort articles by filename */
    articles.sort_by(|a, b| { a.filename.cmp(&b.filename) });

    /* Render index */
    let index_template = load_file("template/index.html")?;
    let index_engine = Handlebars::new();
    
    let ctx = Context {
        title: "A tech blog".to_string(),
        articles: &articles
    };
    
    let out_dir = PathBuf::from("public");

    /* Clean out dir */
    clean_dir(&out_dir)?;

    let index_html = index_engine.render_template(&index_template, &ctx)?;
    write_file(out_dir.join("index.html"), &index_html)?;

    /* Render each article */
    fs::create_dir_all(&out_dir)?;

    let article_template = load_file("template/article.html")?;
    let mut article_engine = Handlebars::new();
    article_engine.register_escape_fn(handlebars::no_escape);
    article_engine.register_template_string("main", &article_template)?;

    for art in &articles {
        let article_html = article_engine.render("main", &art)?;
        let output_file = out_dir.join(&art.filename);
        write_file(output_file, &article_html)?;
    }

    /* Copy contents of `template` dir into output dir, but skip html files */
    copy_dir(Path::new("template"), &out_dir)?;
    Ok(())
}
