use std::fs;
use std::path::PathBuf;
use lazy_static::lazy_static;
use regex::Regex;
use yaml_rust::{YamlLoader};
use chrono::{NaiveDate};
use pulldown_cmark::{Options, Parser};
use tera::{Context, Tera};

use crate::html;

#[derive(Debug,Default)]
pub struct Blog {
    // front matter
    pub code:       String,
    pub title:      String,
    pub desc:       String,
    pub template:   String,
    pub target:     String,
    pub date:       String,
    pub tags:       Vec<String>,

    // markdown content
    pub raw:        String,
}

lazy_static! {
    static ref INFO_RE: Regex =
        Regex::new(r"^[[:space:]]*<!\-\-\r?\n((?s).*?(?-s))\-\->\r?\n?((?s).*(?-s))$").unwrap();
}

pub fn parse(path: &PathBuf) -> Result<Blog, String> {
    let all_content = fs::read_to_string(path.to_str().unwrap()).expect("Can't open file");
    if !INFO_RE.is_match(&all_content) {
        panic!(format!("Can't find front matter inside markdown for {}", path.to_str().unwrap()));
    }

    // extract front matter and raw
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    let caps = INFO_RE.captures(&all_content).unwrap();
    let front_doc = &YamlLoader::load_from_str(&caps[1]).unwrap()[0];
    let raw = caps[2].to_string();

    // fill blog's info and content
    let mut blog: Blog = Default::default();
    blog.code = front_doc["code"].as_str().unwrap().to_string();
    blog.title = front_doc["title"].as_str().unwrap().to_string();
    blog.desc = front_doc["desc"].as_str().unwrap().to_string();
    blog.template = front_doc["template"].as_str().unwrap().to_string();
    blog.target = front_doc["target"].as_str().unwrap().to_string();
    if NaiveDate::parse_from_str( front_doc["date"].as_str().unwrap(), "%Y-%m-%d").is_ok() {
        blog.date = front_doc["date"].as_str().unwrap().to_string();
    } else {
        panic!( format!("Can't parse date from front matter of  {}", path.to_str().unwrap()));
    }
    if !front_doc["tags"].is_badvalue() {
        let tags = front_doc["tags"].as_vec().unwrap();
        for tag in tags {
            blog.tags.push( tag.as_str().unwrap().to_string() );
        }
    }
    blog.raw = raw;

    return Ok(blog);
}

pub fn convert(mkdoc: &str) -> String {
    let parser = Parser::new_ext(mkdoc, Options::empty());

    let mut htdoc = String::new();
    html::push_html(&mut htdoc, parser);
    htdoc
}

pub fn render(tera: &mut tera::Tera, htdoc: &str, blog: &Blog) -> Result<String, String> {
    let mut context = Context::new();

    context.insert("title", &blog.title);
    context.insert("desc", &blog.desc);
    context.insert("markdown", htdoc);

    let result = tera.render("blog.html", &context);
    if result.is_err() {
        return Err(format!("render error: {}", result.err().unwrap().to_string()));
    }

    return Ok( result.unwrap() );
}

