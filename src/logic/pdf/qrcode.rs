use std::io;

use chrono::NaiveDate;
use qrcode_generator::QrCodeEcc;

use log::debug;

pub struct QrCode;

impl QrCode {
    pub fn get(price: f32, currency: &str, account: &str, bank_code: u16, vs: &str, due_date: NaiveDate) -> Result<Vec<u8>, io::Error> {
        let iban = "CZ8830300000001311532017"; // TODO hard code value

        let payment_string = format!("SPD*1.0*ACC:{}*AM:{:.2}*CC:{}*X-VS:{}", iban, price, currency, vs,);

        debug!("Payment string: {}", payment_string);

        qrcode_generator::to_image(payment_string, QrCodeEcc::Low, 256)
    }
}
