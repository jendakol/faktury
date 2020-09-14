use err_context::AnyError;

pub fn create(country_code: &str, account_prefix: Option<u64>, account: u64, bank_code: u16) -> Result<String, AnyError> {
    match country_code {
        "CZ" | "SK" => {
            let account_prefix = account_prefix.unwrap_or_default();

            let mut iban = format!("{:04}{:06}{:010}{}00", bank_code, account_prefix, account, country_code);
            let chsm = checksum(&iban);
            iban.truncate(20); // strip "CZ00" suffix

            Ok(format!("{}{}{}", country_code, chsm, iban))
        }
        _ => Err(AnyError::from(format!("Unsupported country code: {}", country_code))),
    }
}

fn checksum(iban: &str) -> u8 {
    // This code is more-or-less copied from iban_validate crate
    let modulo = iban.as_bytes().iter().fold(0, |acc, c| {
        // Convert '0'-'Z' to 0-35
        let digit = (*c as char)
            .to_digit(36)
            .expect("An address was supplied to compute_checksum with an invalid character.");
        // If the number consists of two digits, multiply by 100
        let multiplier = if digit > 9 { 100 } else { 10 };
        // Calculate modulo
        (acc * multiplier + digit) % 97
    });

    98 - modulo as u8
}

#[cfg(test)]
mod tests {
    use iban::Iban;

    use crate::logic::iban::create;

    #[test]
    fn basic1() {
        let iban_expected = "CZ6508000000192000145399";

        let iban_created = create("CZ", Some(19), 2000145399, 800).unwrap();

        assert_eq!(iban_expected, iban_created.as_str());
        iban_created.parse::<Iban>().unwrap(); // check it's valid
    }

    #[test]
    fn basic2() {
        let iban_expected = "CZ6230300000001559929018";

        let iban_created = create("CZ", None, 1559929018, 3030).unwrap();

        assert_eq!(iban_expected, iban_created.as_str());
        iban_created.parse::<Iban>().unwrap(); // check it's valid
    }

    #[test]
    fn basic3() {
        let iban_expected = "CZ8830300000001311532017";

        let iban_created = create("CZ", None, 1311532017, 3030).unwrap();

        assert_eq!(iban_expected, iban_created.as_str());
        iban_created.parse::<Iban>().unwrap(); // check it's valid
    }

    #[test]
    fn basic4_sk() {
        let iban_expected = "SK3112000000198742637541";

        let iban_created = create("SK", Some(19), 8742637541, 1200).unwrap();

        assert_eq!(iban_expected, iban_created.as_str());
        iban_created.parse::<Iban>().unwrap(); // check it's valid
    }
}

// TODO fuzztest?
