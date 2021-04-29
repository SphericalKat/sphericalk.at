#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rust_embed;

use std::{ffi::OsStr, io::Cursor, path::PathBuf};

use askama::Template;
use chrono::{Datelike, Local};
use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions};
use rocket::{
    http::{ContentType, Status},
    response,
};

#[derive(RustEmbed)]
#[folder = "public/"]
struct Static;

#[derive(RustEmbed)]
#[folder = "posts/"]
struct Posts;

#[derive(Template)]
#[template(path = "index/index.html")]
struct IndexTemplate {
    year: String,
}

struct Post {
    date: String,
    title: String,
    slug: String,
}

#[derive(Template)]
#[template(path = "blog/index.html")]
struct BlogTemplate {
    year: String,
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "blog/post.html")]
struct PostTemplate {
    year: String,
    post: String,
}

#[get("/")]
fn index() -> IndexTemplate {
    IndexTemplate {
        year: Local::now().date().year().to_string(),
    }
}

#[get("/blog")]
fn blog() -> BlogTemplate {
    let post_list: Vec<_> = Posts::iter()
        .map(|f| {
            let slug = f.as_ref();
            let split: Vec<_> = slug.splitn(2, '_').collect();
            println!("{}", slug.to_owned().replace(".md", ""));
            Post {
                date: split[0].to_owned(),
                title: split[1].replace("-", " ").replace(".md", ""),
                slug: slug.to_owned().replace(".md", ""),
            }
        })
        .collect();

    BlogTemplate {
        year: Local::now().date().year().to_string(),
        posts: post_list,
    }
}

#[get("/blog/<file>")]
fn get_blog<'r>(file: String) -> response::Result<'r> {
    let filename = format!("{}.md", file);
    Posts::get(&filename).map_or_else(
        || Err(Status::NotFound),
        |d| {
            let post_text = String::from_utf8(d.as_ref().to_vec()).unwrap();
            let mut opts = &mut ComrakOptions::default();
            opts.extension = ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: false,
                table: true,
                autolink: true,
                tasklist: true,
                superscript: false,
                header_ids: Some("#".to_string()),
                footnotes: false,
                description_lists: false,
                front_matter_delimiter: None,
            };
            let html = markdown_to_html(&post_text, opts);
            response::Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(
                    PostTemplate {
                        year: Local::now().date().year().to_string(),
                        post: html,
                    }
                    .render()
                    .unwrap(),
                ))
                .ok()
        },
    )
}

#[get("/static/<file..>")]
fn public<'r>(file: PathBuf) -> response::Result<'r> {
    let filename = file.display().to_string();
    Static::get(&filename).map_or_else(
        || Err(Status::NotFound),
        |d| {
            let ext = file
                .as_path()
                .extension()
                .and_then(OsStr::to_str)
                .ok_or_else(|| Status::new(400, "Could not get file extension"))?;
            let content_type = ContentType::from_extension(ext)
                .ok_or_else(|| Status::new(400, "Could not get file content type"))?;
            response::Response::build()
                .header(content_type)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}

fn main() {
    rocket::ignite()
        .mount("/", routes!(index, public, blog, get_blog))
        .launch();
}
