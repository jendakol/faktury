use std::fs::File;
use std::io::BufWriter;
use std::iter::FromIterator;
use std::net::SocketAddr;
use std::{io, thread};

use actix_web::body::{Body, BodyStream};
use actix_web::web::Bytes;
use actix_web::{get, head, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use futures::channel::mpsc;
use futures::sink::Sink;
use futures::{executor, SinkExt, StreamExt};
use log::{debug, info};
use printpdf::*;
use qrcode_generator::QrCodeEcc;

fn get_pdf(output: &mut dyn io::Write) {
    let qrcode = qrcode_generator::to_image("Zadek!!!", QrCodeEcc::Low, 512).unwrap();

    let (doc, page1, layer1) = PdfDocument::new("Faktura", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let points1 = vec![
        (Point::new(Mm(20.0), Mm(297.0 - 20.0)), false),
        (Point::new(Mm(20.0), Mm(297.0 - 50.0)), false),
        (Point::new(Mm(80.0), Mm(297.0 - 50.0)), false),
        (Point::new(Mm(80.0), Mm(297.0 - 20.0)), false),
    ];

    let points2 = vec![
        (Point::new(Mm(210.0 - 20.0), Mm(297.0 - 20.0)), false),
        (Point::new(Mm(210.0 - 20.0), Mm(297.0 - 50.0)), false),
        (Point::new(Mm(210.0 - 80.0), Mm(297.0 - 50.0)), false),
        (Point::new(Mm(210.0 - 80.0), Mm(297.0 - 20.0)), false),
    ];

    let line1 = Line {
        points: points1,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };

    let line2 = Line {
        points: points2,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };

    let fill_color = Color::Rgb(Rgb::new(0.25, 0.25, 0.25, None));
    let outline_color = Color::Rgb(Rgb::new(0.15, 0.0, 0.55, None));
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(20);

    current_layer.set_fill_color(fill_color);
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(5.0);

    current_layer.add_shape(line1);
    current_layer.add_shape(line2);

    let image_file_2 = ImageXObject {
        width: Px(512),
        height: Px(512),
        color_space: ColorSpace::Greyscale,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: qrcode,
        image_filter: None,
        clipping_bbox: None,
    };

    let image2 = Image::from(image_file_2);

    image2.add_to_layer(
        current_layer.clone(),
        Some(Mm(20.0)),
        Some(Mm(50.0)),
        None,
        None,
        None,
        None,
    );

    doc.save(&mut BufWriter::new(output)).unwrap();
}

struct MyWrite<T>(mpsc::Sender<T>);

impl<T> io::Write for MyWrite<T>
where
    T: for<'a> From<&'a [u8]> + Send + Sync + 'static,
{
    fn write(&mut self, d: &[u8]) -> io::Result<usize> {
        let len = d.len();

        futures::executor::block_on(self.0.send(d.into()))
            .map(|()| len)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> io::Result<()> {
        futures::executor::block_on(self.0.flush())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

fn foo() -> impl futures::Stream<Item = Vec<u8>> {
    let (tx, rx) = mpsc::channel(5);

    let mut w = MyWrite(tx);

    thread::spawn(move || get_pdf(&mut w));

    rx
}

#[get("/download")]
async fn get() -> impl Responder {
    let stream = foo();
    let stream = stream.map(Bytes::from).map(Ok::<Bytes, ()>);

    HttpResponse::Ok().body(BodyStream::new(stream)).await
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("Starting server on {}", addr);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .service(get)
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(addr)?
    .run()
    .await
}
