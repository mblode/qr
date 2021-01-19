use std::env;
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, guard};
use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use qrcode::QrCode;
use qrcode::render::svg;
use std::fs::File;
use std::io::prelude::*;

const MAX_SIZE: usize = 4096;

#[derive(Debug, Serialize, Deserialize)]
struct QrObject {
    // Basic
    pub qr_code_text: String,

    #[serde(default = "Extension::png")]
    pub image_format: Extension,

    #[serde(default = "default_image_width")]
    pub image_width: u32,

    #[serde(default)]
    pub download: bool,

    // Design
    #[serde(default = "default_color_black")]
    pub foreground_color: String,
    
    #[serde(default = "default_color_white")]
    pub background_color: String,
    
    // Marker Left
    #[serde(default = "default_color_black")]
    pub marker_left_inner_color: String,
    
    #[serde(default = "default_color_black")]
    pub marker_left_outer_color: String,

    #[serde(default = "Marker::default")]
    pub marker_left_template: Marker,

    // Marker Right
    #[serde(default = "default_color_black")]
    pub marker_right_inner_color: String,
    
    #[serde(default = "default_color_black")]
    pub marker_right_outer_color: String,

    #[serde(default = "Marker::default")]
    pub marker_right_template: Marker,

    // Marker Top
    #[serde(default = "default_color_black")]
    pub marker_top_inner_color: String,
    
    #[serde(default = "default_color_black")]
    pub marker_top_outer_color: String,

    #[serde(default = "Marker::default")]
    pub marker_top_template: Marker,
    
    // Marker Bottom
    #[serde(default = "default_color_black")]
    pub marker_bottom_inner_color: String,
    
    #[serde(default = "default_color_black")]
    pub marker_bottom_outer_color: String,

    #[serde(default = "Marker::default")]
    pub marker_bottom_template: Marker,

    // Shape and Logo
    #[serde(default = "Shape::square")]
    pub qr_code_shape: Shape,

    #[serde(default = "Logo::scan")]
    pub qr_code_logo: Logo,

    // Frame
    #[serde(default = "default_color_black")]
    pub frame_color: String,

    #[serde(default = "default_frame_text")]
    pub frame_text: String,

    #[serde(default = "default_color_white")]
    pub frame_text_color: String,
    
    #[serde(default = "Frame::bottom")]
    pub frame_name: Frame,
}

#[derive(Serialize, Deserialize, Debug)]
enum Extension { PNG, SVG }
impl Extension {
    fn png() -> Self { Extension::PNG }
}

#[derive(Serialize, Deserialize, Debug)]
enum Frame { NoFrame, TopHeader, BottomFrame, BottomTooltip }
impl Frame {
    fn bottom() -> Self { Frame::BottomFrame }
}

#[derive(Serialize, Deserialize, Debug)]
enum Marker { Version1, Version2, Version3, Version4, Version5 }
impl Marker {
    fn default() -> Self { Marker::Version1 }
}

#[derive(Serialize, Deserialize, Debug)]
enum Logo { NoLogo, ScanMeSquare, ScanMe }
impl Logo {
    fn scan() -> Self { Logo::ScanMe }
}

#[derive(Serialize, Deserialize, Debug)]
enum Shape { Square, Rounded, Circles }
impl Shape {
    fn square() -> Self { Shape::Square }
}

fn default_frame_text() -> String {
    "Scan me".to_string()
}

fn default_color_white() -> String {
    "#ffffff".to_string()
}

fn default_color_black() -> String {
    "#000000".to_string()
}

fn default_image_width() -> u32 {
    500
}

async fn create(info: web::Json<QrObject>, req: HttpRequest) -> String {
    println!("request: {:?}", req);
    println!("model: {:?}", info);

    let code = QrCode::new(info.qr_code_text.as_bytes()).unwrap();
    let image = code.render()
        .min_dimensions(info.image_width, info.image_width)
        .dark_color(svg::Color(info.foreground_color.as_str()))
        .light_color(svg::Color(info.background_color.as_str()))
        .build();

    let mut file = File::create("output.svg").unwrap();
    file.write_all(image.as_bytes()).unwrap();

    image

}

/// 404 handler
async fn p404() -> &'static str {
    "Page not found!\r\n"
}

#[get("/")]
async fn index() -> &'static str {
    "Hello world!\r\n"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_todo=debug,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(MAX_SIZE)) // <- limit size of the payload (global configuration)

            // index
            .service(index)

            // Create QR code
            .service(web::resource("/create")
                // change json extractor configuration
                .route(web::post().to(create)))

            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )

    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
