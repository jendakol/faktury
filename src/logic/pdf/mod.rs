use actix_web::web::Bytes;
use chrono::NaiveDate;
use err_context::AnyError;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use itertools::Itertools;
use log::{debug, trace};
use printpdf::image_crate::bmp::BmpDecoder;
use printpdf::*;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::{io, thread};

use crate::dao::{Contact, Entrepreneur, Invoice, InvoiceRow, Vat};
use crate::logic::pdf::qrcode::QrCode;
use crate::logic::settings::AccountSettings;

// const PAPER_HEIGHT: f64 = 297.0;
const PAPER_WIDTH: f64 = 210.0;
const PAPER_BORDER: f64 = 20.0;
const LINE_SPACE: f64 = 5.25;

const WIDTH_NUMBER: f64 = 1.8;
const WIDTH_SPACE: f64 = 0.75;
const WIDTH_DOT: f64 = 0.8;
const WIDTH_CURR_SYMBOL: f64 = 3.2; // TODO for hardcoded Kč

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
    ) -> impl futures::Stream<Item = Result<Bytes, Infallible>> {
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

        rx.map(Bytes::from).map(Ok::<Bytes, Infallible>)
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
        settings: AccountSettings,
        entrepreneur: Entrepreneur,
        contact: Contact,
        invoice: Invoice,
        invoice_rows: Vec<InvoiceRow>,
    ) -> Result<PdfDocumentReference, AnyError> {
        trace!("Using account settings: {:?}", settings);

        let font_calibri_light = self.load_font("CalibriLight")?;
        let font_calibri_bold = self.load_font("CalibriBold")?;

        let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));

        self.current_layer.set_outline_color(outline_color);
        self.current_layer.set_outline_thickness(0.25);

        // ----

        self.current_layer // TODO hard code value
            .use_text("faktura", 44.0, Mm(PAPER_BORDER), Mm(266.0), &font_calibri_bold);

        self.current_layer
            .use_text(&invoice.code, 10.0, Mm(PAPER_BORDER), Mm(260.5), &font_calibri_light);

        self.current_layer.use_text(
            invoice.created.format("%d.%m.%Y").to_string(),
            10.0,
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

        if settings.invoice.show_lawyerbox_handover {
            self.lawyer_bullshit_box2(132.5, offset_bottom - 2.0 * LINE_SPACE, &font_calibri_light)?;
        }

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

    #[allow(clippy::too_many_arguments)]
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

        layer.use_text(header, 10.0, Mm(offset_left), Mm(offset_bottom), font);
        offset_bottom -= 2.0 * LINE_SPACE;

        for line in name.split("\r\n") {
            offset_bottom -= LINE_SPACE;
            layer.use_text(line, 10.0, Mm(offset_left), Mm(offset_bottom), font_bold)
        }

        for line in addr.split("\r\n") {
            offset_bottom -= LINE_SPACE;
            layer.use_text(line, 10.0, Mm(offset_left), Mm(offset_bottom), font)
        }

        // ičo
        offset_bottom -= 2.0 * LINE_SPACE;
        let code = match code {
            None => String::new(),
            Some(code) => format!("IČO {}", code), // TODO hard code value
        };
        layer.use_text(code, 10.0, Mm(offset_left), Mm(offset_bottom), font);

        // dič
        offset_bottom -= LINE_SPACE;
        let vat = match vat {
            Vat::Code(code) => Some(format!("DIČ {}", code)),    // TODO hard code value
            Vat::NotTaxPayer => Some("Neplátce DPH".to_owned()), // TODO hard code value
            Vat::DontDisplay => None,
        };

        if let Some(vat) = vat {
            layer.use_text(vat, 10.0, Mm(offset_left), Mm(offset_bottom), font);
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
            let phone = PdfCreator::split_phone_parts(phone);

            layer.use_text(phone, 8.0, Mm(offset_left + 5.6), Mm(offset_bottom), font);
            self.add_img("icon_phone.bmp", offset_left, offset_bottom - 1.0)?;
            offset_bottom -= LINE_SPACE;
        }

        if let Some(email) = email {
            layer.use_text(email, 8.0, Mm(offset_left + 5.6), Mm(offset_bottom), font);
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
            8.0,
            Mm(offset_left),
            Mm(offset_bottom),
            font,
        );
        offset_bottom -= LINE_SPACE;
        layer.use_text(
            "Úřad příslušný podle § 71 odst. 2 živnostenského", // TODO hard code value
            8.0,
            Mm(offset_left),
            Mm(offset_bottom),
            font,
        );
        offset_bottom -= LINE_SPACE; // TODO hard code value
        layer.use_text(format!("zákona: {}.", office_place), 8.0, Mm(offset_left), Mm(offset_bottom), font);

        Ok(())
    }

    fn lawyer_bullshit_box2(&self, offset_left: f64, offset_bottom: f64, font: &IndirectFontRef) -> Result<(), AnyError> {
        let mut offset_bottom = offset_bottom;

        let layer = &self.current_layer;

        // TODO hard code value
        layer.use_text("Předáno dne:", 8.0, Mm(offset_left), Mm(offset_bottom), font);
        offset_bottom -= LINE_SPACE;
        layer.use_text("Převzal:", 8.0, Mm(offset_left), Mm(offset_bottom), font);

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

        let (total_price, use_decs) = invoice_rows.iter().fold((0f64, false), |(tp, decs), r| {
            let row_price = r.item_count as f64 * r.item_price as f64;
            let decs = decs || row_price % 1.0 != 0.0;
            (tp + row_price, decs)
        });

        let line = vec![
            (Point::new(Mm(offset_left), Mm(offset_bottom + 6.0)), false),
            (Point::new(Mm(PAPER_WIDTH - PAPER_BORDER), Mm(offset_bottom + 6.0)), false),
        ];

        let total_price_formatted = PdfCreator::format_price(total_price, use_decs);

        layer.use_text(
            format!("{} Kč", total_price_formatted), // TODO hard code value
            10.0,
            Mm(PAPER_WIDTH - PAPER_BORDER - Self::price_width(&total_price_formatted) - WIDTH_SPACE - WIDTH_CURR_SYMBOL),
            Mm(offset_bottom),
            font_bold,
        );
        offset_bottom += 2.0 * 3.0 + LINE_SPACE;

        invoice_rows.reverse();

        for row in invoice_rows {
            let price = row.item_count as f64 * row.item_price as f64;

            let mut item_name_rows = row.item_name.split("\r\n").collect_vec();
            item_name_rows.reverse(); // because rows are rendered from bottom

            let mut base_row = true;
            for item_name_row in item_name_rows {
                layer.use_text(item_name_row, 10.0, Mm(offset_left), Mm(offset_bottom), font);

                if base_row {
                    let price_formatted = PdfCreator::format_price(price, use_decs);

                    let left_align = PAPER_WIDTH - PAPER_BORDER - Self::price_width(&price_formatted) - WIDTH_SPACE - WIDTH_CURR_SYMBOL;

                    layer.use_text(
                        format!("{} Kč", price_formatted), // TODO hard code value
                        10.0,
                        Mm(left_align),
                        Mm(offset_bottom),
                        font,
                    );

                    base_row = false;
                }

                offset_bottom += LINE_SPACE;
            }
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

    #[allow(clippy::too_many_arguments)]
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
        layer.use_text("PLATEBNÍ ÚDAJE", 10.0, Mm(offset_left), Mm(offset_bottom), font);

        offset_bottom -= 2.0 * LINE_SPACE;

        // TODO hard code value
        layer.use_text("zaplaťte prosím na účet č.", 10.0, Mm(offset_left), Mm(offset_bottom), font);
        layer.use_text(
            format!("{}/{:04}", account_no, bank_code),
            10.0,
            Mm(offset_left + 37.0),
            Mm(offset_bottom),
            font_bold,
        );

        offset_bottom -= LINE_SPACE;

        // TODO hard code value
        layer.use_text("s variabilním symbolem", 10.0, Mm(offset_left), Mm(offset_bottom), font);
        layer.use_text(vs, 10.0, Mm(offset_left + 34.5), Mm(offset_bottom), font_bold);

        offset_bottom = PAPER_BORDER;

        // TODO hard code value
        layer.use_text("do", 10.0, Mm(offset_left), Mm(offset_bottom), font);
        layer.use_text(
            due_date.format("%d.%m.%Y").to_string(),
            10.0,
            Mm(offset_left + 5.0),
            Mm(offset_bottom),
            font_bold,
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
            ImageTransform {
                translate_x: Some(Mm(PAPER_BORDER - 1.2)),
                translate_y: Some(Mm(PAPER_BORDER - 1.2)),
                rotate: None,
                scale_x: None,
                scale_y: None,
                dpi: Some(2.54 / (qr_size / 10.0) * 256.0),
            },
        );

        Ok(())
    }

    fn add_img(&self, path: &str, x: f64, y: f64) -> Result<(), AnyError> {
        let mut image_file = File::open(format!("imgs/{}", path))?;
        let image = Image::try_from(BmpDecoder::new(&mut image_file)?)?;

        image.add_to_layer(
            self.current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                rotate: None,
                scale_x: None,
                scale_y: None,
                dpi: Some(2.54 / (4.0 / 10.0) * 113.0),
            },
        );

        Ok(())
    }

    fn price_width(formatted: &str) -> f64 {
        formatted.chars().fold(0f64, |sum, c| match c {
            '.' => sum + WIDTH_DOT,
            ' ' => sum + WIDTH_SPACE,
            _ => sum + WIDTH_NUMBER,
        })
    }

    fn split_phone_parts(phone: &str) -> String {
        let mut tmp = Vec::new();

        for (c, i) in phone.chars().rev().zip(1..=phone.len()) {
            // remove last space of this is the trailing +
            if c == '+' && i % 3 == 1 {
                tmp.pop();
            }

            tmp.push(c);

            // add space each after each third char
            if i % 3 == 0 && c != '+' {
                tmp.push(' ');
            }
        }

        tmp.reverse();
        tmp.iter().collect()
    }

    fn format_price(price: f64, use_decimals: bool) -> String {
        let prec: usize = if use_decimals { 2 } else { 0 };
        let places: usize = if use_decimals { 6 } else { 3 }; // 000.00 == 6 chars

        if price < 1_000f64 {
            format!("{:.prec$}", price, prec = prec)
        } else if price < 1_000_000f64 {
            format!("{} {:0p$.prec$}", price as u64 / 1_000, price % 1_000f64, prec = prec, p = places)
        } else {
            let mils = price as u64 / 1_000_000;
            let thousands = (price as u64 % 1000000) / 1_000;
            let rest = price % 1_000f64;

            format!("{} {:03} {:0p$.prec$}", mils, thousands, rest, prec = prec, p = places)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_phone_parts() {
        assert_eq!(PdfCreator::split_phone_parts("+420123456789"), String::from("+420 123 456 789"));
        assert_eq!(PdfCreator::split_phone_parts("+44123456789"), String::from("+44 123 456 789"));
        assert_eq!(PdfCreator::split_phone_parts("+1123456789"), String::from("+1 123 456 789"));
        assert_eq!(PdfCreator::split_phone_parts("+1123456789123"), String::from("+1 123 456 789 123"));
        assert_eq!(
            PdfCreator::split_phone_parts("+11234567891234"),
            String::from("+11 234 567 891 234")
        );
    }

    #[test]
    fn test_format_price() {
        assert_eq!(PdfCreator::format_price(1.0, false), "1");
        assert_eq!(PdfCreator::format_price(1.0, true), "1.00");

        assert_eq!(PdfCreator::format_price(100.0, false), "100");
        assert_eq!(PdfCreator::format_price(100.0, true), "100.00");

        assert_eq!(PdfCreator::format_price(1000.0, false), "1 000");
        assert_eq!(PdfCreator::format_price(1000.1, true), "1 000.10");
        assert_eq!(PdfCreator::format_price(1000.12, true), "1 000.12");
        assert_eq!(PdfCreator::format_price(1000.12, false), "1 000");

        assert_eq!(PdfCreator::format_price(100000.0, false), "100 000");
        assert_eq!(PdfCreator::format_price(100000.1, true), "100 000.10");
        assert_eq!(PdfCreator::format_price(100000.12, true), "100 000.12");

        assert_eq!(PdfCreator::format_price(1000000.0, false), "1 000 000");
        assert_eq!(PdfCreator::format_price(1000456.1, true), "1 000 456.10");
        assert_eq!(PdfCreator::format_price(1000768.12, true), "1 000 768.12");

        assert_eq!(PdfCreator::format_price(10000000.0, false), "10 000 000");
        assert_eq!(PdfCreator::format_price(10000000.1, true), "10 000 000.10");
        assert_eq!(PdfCreator::format_price(10000000.12, true), "10 000 000.12");
    }
}
