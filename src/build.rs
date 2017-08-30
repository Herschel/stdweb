///! Grabs the latest WebIDL files from the Mozilla source tree, and converts them into a Rust API
///! for stdweb-rs.

extern crate curl;
use curl::easy::Easy;

extern crate tar;
use tar::Archive;

extern crate flate2;
use flate2::read::GzDecoder;

extern crate webidl;
use webidl::*;
use webidl::ast::*;
use webidl::visitor::*;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const DOM_WEBIDL_URL: &str = "https://hg.mozilla.org/mozilla-central/archive/tip.tar.gz/servo/components/script/dom/webidls/";

fn main() {
    let idl_path = Path::new("webidl");
    if !idl_path.exists() {
        let idl_archive_path = Path::new("idl.tar.gz");

        if !idl_archive_path.exists() {
            let mut idl_out = File::create(idl_archive_path).unwrap();
            let mut idl_downloader = Easy::new();
            idl_downloader.url(DOM_WEBIDL_URL).unwrap();
            idl_downloader.write_function(move |data| {
                Ok(idl_out.write(data).unwrap())
            }).unwrap();
            idl_downloader.perform().unwrap();
        }

        let idl_file = File::open(idl_archive_path).unwrap();
        let mut archive = Archive::new(GzDecoder::new(idl_file).unwrap());

        std::fs::create_dir(idl_path).unwrap();

        for file in archive.entries().unwrap() {
            // Make sure there wasn't an I/O error
            let mut file = file.unwrap();
            let file_name = file.path().unwrap().file_name().unwrap().to_owned();
            file.unpack(idl_path.join(file_name)).unwrap();
        }
    }

    for entry in std::fs::read_dir(idl_path).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().is_some() && entry.path().extension().unwrap() == "webidl" {
            if let Ok(idl) = read_idl_from_file(&entry.path()) {
                let rs_src = WebIdlToRsGenerator::generate(&idl);
    //            println!("{}", rs_src);
            } else {
                println!("Path: {}", entry.path().file_name().unwrap().to_string_lossy());
                println!("FAILED: {}", entry.path().to_string_lossy());
            }
        }
    }

    panic!("SUCCESS");
}

fn read_idl_from_file(path: &Path) -> Result<Vec<Definition>, Box<Error>> {
    let mut file = File::open(path)?;

    //println!("Path: {}", path.file_name().unwrap().to_string_lossy());

    let mut src = String::new();
    file.read_to_string(&mut src)?;
    
    let parser = Parser::new();
    let idl = parser.parse_string(&src).map_err(|e| {
        println!("FAILED: {}", path.file_name().unwrap().to_string_lossy());
        println!("{:?}", e);
        panic!();
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Error")
    })?;

    Ok(idl)
}

struct WebIdlToRsGenerator(String);

impl WebIdlToRsGenerator {
    fn generate(definitions: &[Definition]) -> String {
        let mut visitor = WebIdlToRsGenerator(String::new());
        visitor.visit(definitions);
        visitor.0
    }
}

impl<'ast> ImmutableVisitor<'ast> for WebIdlToRsGenerator {

}
