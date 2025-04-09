use actix_files::{Files, NamedFile};
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, Result};
use actix_multipart::Multipart;
use actix_cors::Cors;
use futures_util::StreamExt as _;
use lopdf::Document;
use printpdf::{PdfDocument, Mm, BuiltinFont};
use regex::Regex;
use sanitize_filename::sanitize;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use tempfile::tempdir;

#[get("/download")]
async fn download_file() -> Result<NamedFile> {
    match NamedFile::open("./tmp/invoice_output.pdf") {
        Ok(file) => Ok(file),
        Err(e) => {
            eprintln!("Failed to open output file: {}", e);
            Err(actix_web::error::ErrorNotFound("File not found"))
        }
    }
}

async fn upload(mut payload: Multipart) -> Result<impl Responder> {
    // Create temp directory if it doesn't exist
    let tmp_dir = tempdir()?;
    let tmp_path = tmp_dir.path();
    std::fs::create_dir_all("./tmp")?;

    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error processing form field: {}", e);
                continue;
            }
        };

        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap_or("upload.pdf");
        let sanitized_name = sanitize(filename);
        let filepath = tmp_path.join(&sanitized_name);

        // Create and write to temp file
        let mut f = match File::create(&filepath) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to create temp file: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Failed to process file"));
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error reading chunk: {}", e);
                    return Ok(HttpResponse::InternalServerError().body("Error reading file data"));
                }
            };
            if let Err(e) = f.write_all(&data) {
                eprintln!("Error writing to file: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Error saving file"));
            }
        }

        // Process the PDF
        let doc = match Document::load(&filepath) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Error loading PDF: {}", e);
                return Ok(HttpResponse::BadRequest().body("Invalid PDF file"));
            }
        };

        let mut full_text = String::new();
        for (page_id, _) in doc.get_pages() {
            if let Ok(text) = doc.extract_text(&[page_id]) {
                full_text.push_str(&text);
            }
        }

        let re = match Regex::new(r"(?m)(?P<name>[\w\s]+)\s*-\s*(?P<pkgs>\d+\s*pkgs)\s*-\s*(?P<cost>\$\d+)\s*-\s*(?P<pcs>\d+\s*pcs)") {
            Ok(re) => re,
            Err(e) => {
                eprintln!("Regex compilation failed: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Internal server error"));
            }
        };

        let mut items = Vec::new();
        for caps in re.captures_iter(&full_text) {
            items.push((
                caps["name"].trim().to_string(),
                caps["pkgs"].to_string(),
                caps["cost"].to_string(),
                caps["pcs"].to_string(),
                "HS000000".to_string(),
            ));
        }

        if let Err(e) = create_output_pdf(items) {
            eprintln!("PDF creation failed: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to generate output PDF"));
        }

        // Temp files will be automatically deleted when tmp_dir goes out of scope
        return Ok(HttpResponse::Ok().body(
            r#"<h2>Invoice processed successfully!</h2>
               <a href="/download">Download processed invoice (PDF)</a>"#,
        ));
    }

    Ok(HttpResponse::BadRequest().body("No file uploaded"))
}

fn create_output_pdf(items: Vec<(String, String, String, String, String)>) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice Output", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = Mm(270.0);

    current_layer.use_text("Parsed Invoice Summary", 16.0, Mm(15.0), y, &font);
    y -= Mm(10.0);

    for (name, pkgs, cost, pcs, hs) in items {
        let line = format!("{:<20} | {:<10} | {:<10} | {:<10} | {:<10}", name, pkgs, cost, pcs, hs);
        current_layer.use_text(&line, 10.0, Mm(15.0), y, &font);
        y -= Mm(8.0);
    }

    let output_path = PathBuf::from("./tmp/invoice_output.pdf");
    let output_file = File::create(&output_path)?;
    doc.save(&mut BufWriter::new(output_file))?;
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Ensure tmp directory exists
    std::fs::create_dir_all("./tmp")?;

    HttpServer::new(|| {
        App::new()
            .app_data(web::PayloadConfig::new(10_000_000)) // 10MB limit
            .wrap(Cors::permissive())
            .route("/upload", web::post().to(upload))
            .service(download_file)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}