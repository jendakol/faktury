use std::ops::Deref;

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::dao::Account;
use crate::logic::invoices::InvoiceNamingSchemaType;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountSettings {
    #[serde(default)]
    pub invoice: AccountInvoiceSettings,
}

impl From<&Account> for AccountSettings {
    fn from(account: &Account) -> Self {
        serde_json::from_str(account.settings.as_str())
            .expect("It must never fail - serde has defaults")
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountInvoiceSettings {
    #[serde(default)]
    pub naming_schema: InvoiceNamingSchemaType,
    #[serde(default, with = "default_due_length_serde")]
    pub default_due_length: DefaultDueLength,
}

#[derive(Clone, Debug)]
pub struct DefaultDueLength(Duration);

impl Deref for DefaultDueLength {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DefaultDueLength {
    fn default() -> Self {
        Self(Duration::days(14))
    }
}

mod default_due_length_serde {
    use chrono::Duration;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    use crate::logic::settings::DefaultDueLength;

    pub fn serialize<S>(date: &DefaultDueLength, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = date.0.num_milliseconds();
        millis.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DefaultDueLength, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        Ok(DefaultDueLength(Duration::milliseconds(millis)))
    }
}
