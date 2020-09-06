use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::{io, thread};

use actix_web::web::Bytes;
use chrono::NaiveDate;
use err_context::AnyError;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use log::debug;
use printpdf::*;

use crate::dao::{Contact, Entrepreneur, Invoice, InvoiceRow, Vat};
use crate::logic::pdf::qrcode::QrCode;
use crate::logic::settings::AccountSettings;

// const PAPER_HEIGHT: f64 = 297.0;
const PAPER_WIDTH: f64 = 210.0;
const PAPER_BORDER: f64 = 20.0;
const LINE_SPACE: f64 = 5.25;

mod qrcode;

#[derive(Debug, Clone)]
pub struct PdfManager {
    fonts: Arc<HashMap<String, String>>,
}

impl PdfManager {
    pub fn new() -> Result<Self, AnyError> {
        let mut fonts = HashMap::new();
        fonts.insert(String::from("CalibriLight"), String::from("Calibri Light.ttf"));
        fonts.insert(String::from("CalibriBold"), String::from("Calibri Bold.ttf"));
        fonts.insert(String::from("CalibriLightItalic"), String::from("Calibri Light Italic.ttf"));
        let fonts = Arc::new(fonts);

        Ok(PdfManager { fonts })
    }

    pub fn create(
        &self,
        settings: AccountSettings,
        entrepreneur: Entrepreneur,
        contact: Contact,
        invoice: Invoice,
        invoice_rows: Vec<InvoiceRow>,
    ) -> impl futures::Stream<Item = Result<Bytes, ()>> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
        let w = BlockingWriter(tx);

        let fonts = self.fonts.clone();

        thread::spawn(move || {
            let creator = PdfCreator::from(fonts);
            let doc = creator.create(settings, entrepreneur, contact, invoice, invoice_rows).unwrap();

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

        let doc = doc.with_conformance(PdfConformance::Custom(CustomPdfConformance {
            requires_icc_profile: false,
            requires_xmp_metadata: false,
            ..Default::default()
        }));

        PdfCreator { fonts, doc, current_layer }
    }
}

impl PdfCreator {
    fn load_font(&self, font_name: &str) -> Result<IndirectFontRef, AnyError> {
        let path = format!("fonts/{}", self.fonts.get(font_name).unwrap());
        self.doc
            .add_external_font(File::open(path).map_err(AnyError::from)?)
            .map_err(AnyError::from)
    }

    fn create(
        self,
        _settings: AccountSettings,
        entrepreneur: Entrepreneur,
        contact: Contact,
        invoice: Invoice,
        invoice_rows: Vec<InvoiceRow>,
    ) -> Result<PdfDocumentReference, AnyError> {
        let font_calibri_light = self.load_font("CalibriLight")?;
        let font_calibri_bold = self.load_font("CalibriBold")?;

        let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));

        self.current_layer.set_outline_color(outline_color);
        self.current_layer.set_outline_thickness(0.25);

        // ----

        self.current_layer // TODO hard code value
            .use_text("faktura", 44, Mm(PAPER_BORDER), Mm(266.0), &font_calibri_bold);

        self.current_layer
            .use_text(&invoice.code, 10, Mm(PAPER_BORDER), Mm(260.5), &font_calibri_light);

        self.current_layer.use_text(
            invoice.created.format("%d.%m.%Y").to_string(),
            10,
            Mm(PAPER_BORDER + 30.0),
            Mm(260.5),
            &font_calibri_light,
        );

        // entrepreneur
        let offset_bottom = self.entrepreneur_box(
            65.0,
            230.0,
            &font_calibri_light,
            &font_calibri_bold,
            "DODAVATEL", // TODO hard code value
            &entrepreneur.name,
            &entrepreneur.address,
            Some(&entrepreneur.code),
            &entrepreneur.vat,
        )?;

        let offset_bottom = self.contact_box(
            65.0,
            offset_bottom - 2.0 * LINE_SPACE,
            &font_calibri_light,
            &entrepreneur.phone,
            &entrepreneur.email,
        )?;

        self.lawyer_bullshit_box(65.0, offset_bottom - 2.0 * LINE_SPACE, &font_calibri_light, "Městský úřad Litvínov")?; // TODO hard code value
        self.lawyer_bullshit_box2(132.5, offset_bottom - 2.0 * LINE_SPACE, &font_calibri_light)?;

        // contact
        let _ = self.entrepreneur_box(
            132.5,
            230.0,
            &font_calibri_light,
            &font_calibri_bold,
            "ODBĚRATEL", // TODO hard code value
            &contact.name,
            &contact.address,
            contact.code.as_ref().map(AsRef::as_ref),
            &contact.vat,
        )?;

        let total_price = invoice_rows.iter().fold(0f32, |tp, r| tp + r.item_count as f32 * r.item_price);

        self.rows(65.0, 77.0, &font_calibri_light, &font_calibri_bold, invoice_rows)?;

        self.payment_box(
            65.0,
            PAPER_BORDER + 25.0,
            &font_calibri_light,
            &font_calibri_bold,
            total_price,
            &entrepreneur.currency_code,
            &entrepreneur.account_number_country_code,
            entrepreneur.account_number_prefix.map(|p| p as u64),
            entrepreneur.account_number as u64,
            entrepreneur.account_bank_code as u16,
            &invoice.code,
            invoice.pay_until,
        )?;

        Ok(self.doc)
    }

    fn entrepreneur_box(
        &self,
        offset_left: f64,
        offset_bottom: f64,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        header: &str,
        name: &str,
        addr: &str,
        code: Option<&str>,
        vat: &Vat,
    ) -> Result<f64, AnyError> {
        let mut offset_bottom = offset_bottom;

        let layer = &self.current_layer;

        layer.use_text(header, 10, Mm(offset_left), Mm(offset_bottom), &font);
        offset_bottom -= 2.0 * LINE_SPACE;
        layer.use_text(name, 10, Mm(offset_left), Mm(offset_bottom), &font_bold);

        for line in addr.split("\r\n") {
            offset_bottom -= LINE_SPACE;
            layer.use_text(line, 10, Mm(offset_left), Mm(offset_bottom), &font)
        }

        // ičo
        offset_bottom -= 2.0 * LINE_SPACE;
        let code = match code {
            None => String::new(),
            Some(code) => format!("IČO {}", code), // TODO hard code value
        };
        layer.use_text(code, 10, Mm(offset_left), Mm(offset_bottom), &font);

        // dič
        offset_bottom -= LINE_SPACE;
        let vat = match vat {
            Vat::Code(code) => Some(format!("DIČ {}", code)),    // TODO hard code value
            Vat::NotTaxPayer => Some("Neplátce DPH".to_owned()), // TODO hard code value
            Vat::DontDisplay => None,
        };

        if let Some(vat) = vat {
            layer.use_text(vat, 10, Mm(offset_left), Mm(offset_bottom), &font);
        }

        Ok(offset_bottom)
    }

    fn contact_box(
        &self,
        offset_left: f64,
        offset_bottom: f64,
        font: &IndirectFontRef,
        phone: &Option<String>,
        email: &Option<String>,
    ) -> Result<f64, AnyError> {
        let mut offset_bottom = offset_bottom;

        let layer = &self.current_layer;

        if let Some(phone) = phone {
            layer.use_text(phone, 8, Mm(offset_left + 5.6), Mm(offset_bottom), font);
            self.add_img("icon_phone.bmp", offset_left, offset_bottom - 1.0)?;
            offset_bottom -= LINE_SPACE;
        }

        if let Some(email) = email {
            layer.use_text(email, 8, Mm(offset_left + 5.6), Mm(offset_bottom), font);
            self.add_img("icon_mail.bmp", offset_left, offset_bottom - 1.0)?;
        }

        Ok(offset_bottom)
    }

    fn lawyer_bullshit_box(
        &self,
        offset_left: f64,
        offset_bottom: f64,
        font: &IndirectFontRef,
        office_place: &str,
    ) -> Result<(), AnyError> {
        let mut offset_bottom = offset_bottom;

        let layer = &self.current_layer;

        layer.use_text(
            "Fyzická osoba zapsaná v Živnostenském rejstříku.", // TODO hard code value
            8,
            Mm(offset_left),
            Mm(offset_bottom),
            &font,
        );
        offset_bottom -= LINE_SPACE;
        layer.use_text(
            "Úřad příslušný podle § 71 odst. 2 živnostenského", // TODO hard code value
            8,
            Mm(offset_left),
            Mm(offset_bottom),
            &font,
        );
        offset_bottom -= LINE_SPACE; // TODO hard code value
        layer.use_text(format!("zákona: {}.", office_place), 8, Mm(offset_left), Mm(offset_bottom), &font);

        Ok(())
    }

    fn lawyer_bullshit_box2(&self, offset_left: f64, offset_bottom: f64, font: &IndirectFontRef) -> Result<(), AnyError> {
        let mut offset_bottom = offset_bottom;

        let layer = &self.current_layer;

        // TODO hard code value
        layer.use_text("Předáno dne:", 8, Mm(offset_left), Mm(offset_bottom), &font);
        offset_bottom -= LINE_SPACE;
        layer.use_text("Převzal:", 8, Mm(offset_left), Mm(offset_bottom), &font);

        Ok(())
    }

    fn rows(
        &self,
        offset_left: f64,
        offset_bottom: f64,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        mut invoice_rows: Vec<InvoiceRow>,
    ) -> Result<(), AnyError> {
        let mut offset_bottom = offset_bottom;
        // from bottom up!!!

        let layer = &self.current_layer;

        let total_price = invoice_rows.iter().fold(0f32, |tp, r| tp + r.item_count as f32 * r.item_price);

        let line = vec![
            (Point::new(Mm(offset_left), Mm(offset_bottom + 6.0)), false),
            (Point::new(Mm(PAPER_WIDTH - PAPER_BORDER), Mm(offset_bottom + 6.0)), false),
        ];

        layer.use_text(
            format!("{} {:03} Kč", total_price as u16 / 1000, total_price as u16 % 1000), // TODO hard code value
            10,
            Mm(PAPER_WIDTH - PAPER_BORDER - Self::price_width(total_price)),
            Mm(offset_bottom),
            &font_bold,
        );
        offset_bottom += 2.0 * 3.0 + LINE_SPACE;

        invoice_rows.reverse();

        for row in invoice_rows {
            let price = row.item_count as f32 * row.item_price;

            layer.use_text(&row.item_name, 10, Mm(offset_left), Mm(offset_bottom), &font);
            layer.use_text(
                format!("{} {:03} Kč", price as u16 / 1000, price as u16 % 1000), // TODO hard code value
                10,
                Mm(PAPER_WIDTH - PAPER_BORDER - Self::price_width(price)),
                Mm(offset_bottom),
                &font,
            );
            offset_bottom += LINE_SPACE;
        }

        let line = Line {
            points: line,
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };

        self.current_layer.add_shape(line);

        Ok(())
    }

    fn payment_box(
        &self,
        offset_left: f64,
        mut offset_bottom: f64,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        price: f32,
        currency: &str,
        account_number_country_code: &str,
        account_prefix: Option<u64>,
        account_no: u64,
        bank_code: u16,
        vs: &str,
        due_date: NaiveDate,
    ) -> Result<(), AnyError> {
        let layer = &self.current_layer;

        // TODO hard code value
        layer.use_text("PLATEBNÍ ÚDAJE", 10, Mm(offset_left), Mm(offset_bottom), &font);

        offset_bottom -= 2.0 * LINE_SPACE;

        // TODO hard code value
        layer.use_text("zaplaťte prosím na účet č.", 10, Mm(offset_left), Mm(offset_bottom), &font);
        layer.use_text(
            format!("{}/{}", account_no, bank_code),
            10,
            Mm(offset_left + 37.0),
            Mm(offset_bottom),
            &font_bold,
        );

        offset_bottom -= LINE_SPACE;

        // TODO hard code value
        layer.use_text("s variabilním symbolem", 10, Mm(offset_left), Mm(offset_bottom), &font);
        layer.use_text(vs, 10, Mm(offset_left + 34.5), Mm(offset_bottom), &font_bold);

        offset_bottom = PAPER_BORDER;

        // TODO hard code value
        layer.use_text("do", 10, Mm(offset_left), Mm(offset_bottom), &font);
        layer.use_text(
            due_date.format("%d.%m.%Y").to_string(),
            10,
            Mm(offset_left + 5.0),
            Mm(offset_bottom),
            &font_bold,
        );

        let qrcode = ImageXObject {
            width: Px(256),
            height: Px(256),
            color_space: ColorSpace::Greyscale,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: QrCode::get(
                price,
                currency,
                account_number_country_code,
                account_prefix,
                account_no,
                bank_code,
                vs,
            )?,
            image_filter: None,
            clipping_bbox: None,
        };

        let qr_size = 30.0;

        let qrcode_image = Image::from(qrcode);
        qrcode_image.add_to_layer(
            self.current_layer.clone(),
            Some(Mm(PAPER_BORDER - 1.2)),
            Some(Mm(PAPER_BORDER - 1.2)),
            None,
            None,
            None,
            Some(2.54 / (qr_size / 10.0) * 256.0),
        );

        Ok(())
    }

    fn add_img(&self, path: &str, x: f64, y: f64) -> Result<(), AnyError> {
        let mut image_file = File::open(format!("imgs/{}", path))?;
        let image = Image::try_from(image::bmp::BmpDecoder::new(&mut image_file)?)?;

        image.add_to_layer(
            self.current_layer.clone(),
            Some(Mm(x)),
            Some(Mm(y)),
            None,
            None,
            None,
            Some(2.54 / (4.0 / 10.0) * 113.0),
        );

        Ok(())
    }

    fn price_width(price: f32) -> f64 {
        if price >= 10_000f32 {
            13.8
        } else if price >= 1_000f32 {
            11.8
        } else {
            10.8
        }
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
        futures::executor::block_on(self.0.flush()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
