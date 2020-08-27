use std::io;

use log::debug;
use qrcode_generator::QrCodeEcc;

pub struct QrCode;

impl QrCode {
    pub fn get(price: f32, currency: &str, _country_code: &str, _account: u64, _bank_code: u16, vs: &str) -> Result<Vec<u8>, io::Error> {
        let iban = "CZ8830300000001311532017"; // TODO hard code value

        let payment_string = format!("SPD*1.0*ACC:{}*AM:{:.2}*CC:{}*X-VS:{}", iban, price, currency, vs,);

        debug!("Payment string: {}", payment_string);

        qrcode_generator::to_image(payment_string, QrCodeEcc::Low, 256)
    }
}
