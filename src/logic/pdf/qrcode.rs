use err_context::AnyError;
use log::debug;
use qrcode_generator::QrCodeEcc;

pub struct QrCode;

impl QrCode {
    pub fn get(
        price: f32,
        currency: &str,
        country_code: &str,
        account_prefix: Option<u64>,
        account: u64,
        bank_code: u16,
        vs: &str,
    ) -> Result<Vec<u8>, AnyError> {
        let iban = crate::logic::iban::create(country_code, account_prefix, account, bank_code)?;

        let payment_string = format!("SPD*1.0*ACC:{}*AM:{:.2}*CC:{}*X-VS:{}", iban, price, currency, vs,);

        debug!("Payment string: {}", payment_string);

        Ok(qrcode_generator::to_image(payment_string, QrCodeEcc::Low, 256)?)
    }
}
