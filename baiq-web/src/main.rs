#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

#![feature(custom_derive)]
#![feature(plugin)]

#![plugin(maud_macros)]
#![plugin(rocket_codegen)]

extern crate baimax;
extern crate chrono;
extern crate maud;
extern crate multipart;
extern crate penny;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_json;
extern crate serde;
extern crate serde_json;

use std::io::Read;
use std::path::{Path, PathBuf};

use maud::{Markup, Render};
use multipart::server::Multipart;
use rocket::{Outcome, Request};
use rocket::data::{Data, DataStream, FromData};
use rocket::http::Status;
use rocket::http::uri::URI;
use rocket::response::NamedFile;
use rocket_json::JSON;

#[get("/")]
fn index() -> Markup {
    html! { (NORMALIZE) (STYLE)
        Title("baiq")
        header {
            h1 "baiq"
            h2 "BAI query tool"
        }
        form action="pretty" enctype="multipart/form-data" method="post" target="_blank" {
            fieldset {
                legend "Pretty-print a BAI file"

                div class="control-group" {
                    label for="bai" "File"
                    input required? type="file" name="bai" accept=".bai,.txt" /
                }

                div class="controls" {
                    button type="submit" "Pretty-print"
                }
            }
        }
    }
}

struct MultipartData(Multipart<DataStream>);
impl MultipartData {
    fn new<'a, 'b: 'a>(req: &'a Request<'b>, data: Data) -> Option<Self> {
        req.content_type()
            .and_then(|ct| {
                ct.params()
                    .filter(|&(attr, _)| attr == "boundary")
                    .map(|(_, val)| val)
                    .next()
                    .map(|s| s.to_owned())
            })
            .map(|boundary| {
                MultipartData(Multipart::with_body(data.open(), boundary))
            })
    }
}
impl FromData for MultipartData {
    type Error = ();
    fn from_data(req: &Request, data: Data) -> rocket::data::Outcome<Self, Self::Error> {
        let multipart_ct = rocket::http::ContentType::DataForm;
        let ct = if let Some(ct) = req.content_type() {
            ct
        } else {
            return Outcome::Forward(data);
        };
        if ct.ttype != multipart_ct.ttype || ct.subtype != multipart_ct.subtype {
            return Outcome::Forward(data);
        }

        if let Some(data) = MultipartData::new(req, data) {
            Outcome::Success(data)
        } else {
            Outcome::Failure((Status::BadRequest, ()))
        }
    }
}

struct BaiProcess {
    filename: Option<String>,
    bai: Vec<u8>,
}
impl BaiProcess {
    fn try_from_multipart<R: Read>(data: &mut Multipart<R>) -> Result<BaiProcess, String> {
        let mut bai: Option<BaiProcess> = None;
        while let Some(mut field) = data.read_entry()
            .map_err(|e| format!("Multipart read error: {}.", e))?
        {
            if field.name == "bai" {
                let file = field
                    .data
                    .as_file()
                    .ok_or_else(|| String::from("bai isn't a file"))?;
                let mut buf = Vec::with_capacity(2 << 12);
                file.read_to_end(&mut buf)
                    .map_err(|e| format!("bai reading: {}", e))?;
                bai = Some(BaiProcess {
                    filename: file.filename.clone(),
                    bai: buf,
                });
            }
        }
        bai.ok_or_else(|| String::from("No BAI file provided."))
    }
}
impl FromData for BaiProcess {
    type Error = String;
    fn from_data(req: &Request, data: Data) -> rocket::data::Outcome<Self, Self::Error> {
        let mut data = match MultipartData::from_data(req, data) {
            Outcome::Success(s) => s,
            Outcome::Failure((status, ())) => return Outcome::Failure(
	        (status, String::from("Multipart error."))
	    ),
            Outcome::Forward(f) => return Outcome::Forward(f),
        }.0;
        match BaiProcess::try_from_multipart(&mut data) {
            Ok(bai) => Outcome::Success(bai),
            Err(e) => Outcome::Failure((Status::BadRequest, e)),
        }
    }
}

#[cfg_attr(any(feature = "clippy", feature = "cargo-clippy"), allow(needless_pass_by_value))]
#[post("/json", data = "<bai>")]
fn json(bai: BaiProcess) -> Result<JSON<baimax::data::File>, String> {
    baimax::data::File::process(&bai.bai)
        .map(JSON)
        .map_err(|e| format!("{:?}", e))
}

#[cfg_attr(any(feature = "clippy", feature = "cargo-clippy"), allow(needless_pass_by_value))]
#[post("/pretty", data = "<bai>")]
fn pretty(bai: BaiProcess) -> Result<String, String> {
    baimax::data::File::process(&bai.bai)
        .map(|f| format!("{}", f))
        .map_err(|e| format!("{:?}", e))
}

#[get("/pub/<file..>")]
fn pub_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("pub/").join(file)).ok()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, pretty, json, pub_file])
        .launch();
}

pub struct Css<T: AsRef<str>>(pub T);
const NORMALIZE: Css<&'static str> = Css("/pub/normalize.css");
const STYLE: Css<&'static str> = Css("/pub/style.css");
impl<T: AsRef<str>> Render for Css<T> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" href=(self.0.as_ref()) /
        }
    }
}
pub struct Refresh<'a>(pub u8, pub Option<&'a URI<'a>>);
impl<'a> Render for Refresh<'a> {
    fn render(&self) -> Markup {
        html! {
            @match self.1 {
                None => meta http-equiv="refresh" content=(self.0) /,
                Some(uri) => meta http-equiv="refresh" content={(self.0)";url="(uri)} /,
            }
        }
    }
}
pub struct Title<T: AsRef<str>>(pub T);
impl<T: AsRef<str>> Render for Title<T> {
    fn render(&self) -> Markup {
        html! {
            title (self.0.as_ref())
        }
    }
}
