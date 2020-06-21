use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::{io, thread};

use actix_web::web::Bytes;
use err_context::AnyError;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use log::debug;
use printpdf::*;
use qrcode_generator::QrCodeEcc;

#[derive(Debug, Clone)]
pub struct PdfManager {
    fonts: Arc<HashMap<String, String>>,
}

impl PdfManager {
    pub fn new() -> Result<Self, AnyError> {
        let mut fonts = HashMap::new();
        fonts.insert(String::from("x360"), String::from("X360.ttf"));
        let fonts = Arc::new(fonts);

        Ok(PdfManager { fonts })
    }

    pub fn create(&self, contact_name: String) -> impl futures::Stream<Item = Result<Bytes, ()>> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
        let w = BlockingWriter(tx);

        let fonts = self.fonts.clone();

        thread::spawn(move || {
            let creator = PdfCreator::from(fonts);
            let doc = creator.create(&contact_name).unwrap();

            // TODO how to handle errors? :-(
            if let Err(e) = doc.save(&mut BufWriter::new(w)) {
                debug!("Error while generating PDF: {}", e);
            }
        });

        rx.map(Bytes::from).map(Ok::<Bytes, ()>)
    }
}

struct PdfCreator {
    fonts: Arc<HashMap<String, String>>,
    doc: PdfDocumentReference,
    current_layer: PdfLayerReference,
}

impl From<Arc<HashMap<String, String>>> for PdfCreator {
    fn from(fonts: Arc<HashMap<String, String>>) -> Self {
        let (doc, page1, layer1) = PdfDocument::new("Faktura", Mm(210.0), Mm(297.0), "Layer 1");

        let current_layer = doc.get_page(page1).get_layer(layer1);

        PdfCreator {
            fonts,
            doc,
            current_layer,
        }
    }
}

impl PdfCreator {
    fn load_font(&self, font_name: &str) -> Result<IndirectFontRef, AnyError> {
        self.doc
            .add_external_font(
                File::open(self.fonts.get(font_name).unwrap()).map_err(AnyError::from)?,
            )
            .map_err(AnyError::from)
    }

    fn create(self, contact_name: &str) -> Result<PdfDocumentReference, AnyError> {
        let qrcode = qrcode_generator::to_image("Zadek!!!", QrCodeEcc::Low, 512).unwrap();

        // TODO doesn't work! let font = doc.add_builtin_font(BuiltinFont::TimesRoman).unwrap();
        // TODO doesn't work! let font2 = doc.add_external_font(File::open("texgyreheroscn-regular.otf").unwrap()).unwrap();
        let font = self.load_font("x360")?;

        self.current_layer
            .use_text(contact_name, 20, Mm(100.0), Mm(100.0), &font);

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

        self.current_layer.set_fill_color(fill_color);
        self.current_layer.set_outline_color(outline_color);
        self.current_layer.set_outline_thickness(5.0);

        self.current_layer.add_shape(line1);
        self.current_layer.add_shape(line2);

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
            self.current_layer.clone(),
            Some(Mm(20.0)),
            Some(Mm(50.0)),
            None,
            None,
            None,
            None,
        );

        Ok(self.doc)
    }
}

struct BlockingWriter<T>(mpsc::Sender<T>);

impl<T> io::Write for BlockingWriter<T>
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
