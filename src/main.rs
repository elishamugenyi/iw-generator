use actix_files::{Files, NamedFile};
use actix_web::web::Payload;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse, Result};
use actix_cors::Cors;
use actix_web::http::header;
use lopdf::Document;
use printpdf::{PdfDocument, Pt, Mm, BuiltinFont, Line, Color, Rgb, Point};
//use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::PathBuf;


//receive items array from frontend
/* 
#[derive(Debug, Serialize, Deserialize)]
struct InvoiceRequest {
    items: Vec<InvoiceItem>,
}*/

#[derive(Debug, Serialize, Deserialize)]
struct InvoiceItem {
    description: String,
    packages: String,
    cost: String,
    units: String,
    weight: String,
    hs_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HsCode {
    code: String,
    description: String,
    chapter: String,
    page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchRequest {
    hint: String,
}

#[post("/search-hs-codes")]
async fn search_hs_codes(search: web::Json<SearchRequest>) -> Result<impl Responder> {
    // In a real app, you would load this from your HS code PDF or database
    // This is a simplified example with mock data
    let hs_code = vec![
        HsCode {
            code: "6111.20".to_string(),
            description: "Babies' garments, knitted or crocheted".to_string(),
            chapter: "61".to_string(),
            page: 123,
        },
        HsCode {
            code: "8708.29".to_string(),
            description: "Parts and accessories of motor vehicles".to_string(),
            chapter: "87".to_string(),
            page: 456,
        },
        HsCode {
            code: "7326.90".to_string(),
            description: "Other articles of iron or steel".to_string(),
            chapter: "73".to_string(),
            page: 789,
        },
    ];

    let search_lower = search.hint.to_lowercase();
    let results: Vec<&HsCode> = hs_code
        .iter()
        .filter(|code| {
            code.description.to_lowercase().contains(&search_lower) ||
            code.code.contains(&search.hint)
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "results": results
    })))
}

fn create_output_pdf(items: &[InvoiceItem]) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice Output", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = Mm(270.0);

    // Add title
    current_layer.use_text("PROFORMA INVOICE", 20.0, Mm(15.0), y, &font);
    y -= Mm(15.0);

    // Add headers
    current_layer.use_text("Description", 12.0, Mm(15.0), y, &font);
    current_layer.use_text("Packages", 12.0, Mm(100.0), y, &font);
    current_layer.use_text("Cost (CIF $)", 12.0, Mm(130.0), y, &font);
    current_layer.use_text("Units", 12.0, Mm(160.0), y, &font);
    current_layer.use_text("Weight (kg)", 12.0, Mm(190.0), y, &font);
    y -= Mm(10.0);

    // Draw line
    draw_line(&current_layer, Mm(15.0), y, Mm(195.0), y);
    y -= Mm(10.0);

    // Add items
    for item in items {
        current_layer.use_text(&item.description, 10.0, Mm(15.0), y, &font);
        current_layer.use_text(&item.packages, 10.0, Mm(100.0), y, &font);
        current_layer.use_text(&item.cost, 10.0, Mm(130.0), y, &font);
        current_layer.use_text(&item.units, 10.0, Mm(160.0), y, &font);
        current_layer.use_text(&item.weight, 10.0, Mm(190.0), y, &font);
        y -= Mm(8.0);
    }

    // Save the PDF
    let output_path = PathBuf::from("./tmp/invoice_output.pdf");
    std::fs::create_dir_all("./tmp")?;
    let output_file = File::create(&output_path)?;
    doc.save(&mut BufWriter::new(output_file))?;

    Ok(())
}

#[post("/process-pdf")]
async fn process_pdf(payload: web::Json<Vec<InvoiceItem>>) -> Result<impl Responder> {
    println!("Received items: {:#?}", payload);
    
    match create_output_pdf(&payload.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "PDF processed successfully"
        }))),
        Err(e) => {
            eprintln!("PDF creation failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to generate output PDF: {}", e)
            })))
        }
    }
}
/* 
#[post("/process-pdf")]
async fn process_pdf(items: web::Json<Vec<InvoiceItem>>) -> Result<impl Responder> {
    if let Err(e) = create_output_pdf(&items.into_inner()) {
        eprintln!("PDF creation failed: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "Failed to generate output PDF"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "PDF processed successfully"
    })))
}*/
fn draw_line(layer: &printpdf::PdfLayerReference, x1: Mm, y1: Mm, x2: Mm, y2: Mm) {
    // Create points with explicit Point struct
    let start_point = Point { 
        x: Pt::from(x1),
        y: Pt::from(y1)
    };
    let end_point = Point { 
        x: Pt::from(x2),
        y: Pt::from(y2)
    };
    
    // Create line with correct point types
    let mut line = Line {
        points: vec![
            (start_point, false),  // (Point, is_curve_control)
            (end_point, false),
        ],
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    };
    
    // Correct method names for styling
    //line.set_stroke(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    //line.set_stroke_width(0.5);
    
    layer.add_shape(line);
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp")?;

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::CONTENT_TYPE])
                    .max_age(3600)
            )
            .service(search_hs_codes)
            .service(process_pdf)
            .service(download_file)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}



/* 
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
use serde::Serialize;

#[derive(Serialize)]
struct InvoiceItem {
    name: String,
    pkgs: String,
    cost: String,
    pcs: String,
    hs: String,
}

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
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "Failed to process file"
                })));
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error reading chunk: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Error reading file data"
                    })));
                }
            };
            if let Err(e) = f.write_all(&data) {
                eprintln!("Error writing to file: {}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "Error saving file"
                })));
            }
        }

        // Process the PDF
        let doc = match Document::load(&filepath) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Error loading PDF: {}", e);
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "Invalid PDF file"
                })));
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
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "Internal server error"
                })));
            }
        };

        let mut items = Vec::new();
        for caps in re.captures_iter(&full_text) {
            items.push(InvoiceItem {
                name: caps["name"].trim().to_string(),
                pkgs: caps["pkgs"].to_string(),
                cost: caps["cost"].to_string(),
                pcs: caps["pcs"].to_string(),
                hs: "HS000000".to_string(),
            });
        }

        if items.is_empty() {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "No matching data found in the PDF"
            })));
        }

        if let Err(e) = create_output_pdf(&items) {
            eprintln!("PDF creation failed: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Failed to generate output PDF"
            })));
        }

        // Return the extracted items along with success message
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Invoice processed successfully!",
            "items": items
        })));
    }

    Ok(HttpResponse::BadRequest().json(serde_json::json!({
        "success": false,
        "message": "No file uploaded"
    })))
}

fn create_output_pdf(items: &[InvoiceItem]) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice Output", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = Mm(270.0);

    current_layer.use_text("Parsed Invoice Summary", 16.0, Mm(15.0), y, &font);
    y -= Mm(10.0);

    for item in items {
        let line = format!("{:<20} | {:<10} | {:<10} | {:<10} | {:<10}", 
            item.name, item.pkgs, item.cost, item.pcs, item.hs);
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
}*/