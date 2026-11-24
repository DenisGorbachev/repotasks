use crate::OpItemField;
use age::x25519::Identity;
use errgonomic::handle_opt;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpItem {
    pub title: String,
    #[serde(default)]
    pub fields: Vec<OpItemField>,
}

impl OpItem {
    pub fn age_identity(&self) -> Result<Identity, OpItemAgeIdentityError> {
        use OpItemAgeIdentityError::*;
        let title = &self.title;
        let passwords: Vec<&OpItemField> = self
            .fields
            .iter()
            .filter(|field| field.label.as_deref() == Some("password"))
            .collect();
        let [password] = passwords.as_slice() else {
            return Err(PasswordCountInvalid {
                title: title.into(),
                count: passwords.len(),
            });
        };
        let password = handle_opt!(password.value.as_ref(), PasswordNotFound, title);
        match password.expose_secret().trim().parse() {
            Ok(identity) => Ok(identity),
            Err(reason) => Err(PasswordInvalid {
                title: title.into(),
                reason: reason.into(),
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum OpItemAgeIdentityError {
    #[error("1Password item '{title}' has {count} password fields instead of one")]
    PasswordCountInvalid { title: String, count: usize },
    #[error("the password field of 1Password item '{title}' is empty")]
    PasswordNotFound { title: String },
    #[error("the password of 1Password item '{title}' is not a native age private key: {reason}")]
    PasswordInvalid { title: String, reason: String },
}
