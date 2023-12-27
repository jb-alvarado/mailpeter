use html_parser::Dom;
use lettre::{
    message::header, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};

use crate::utils::errors::ServiceError;
use crate::CONFIG;

#[derive(Debug, Deserialize, Serialize)]
pub struct Msg {
    #[serde(skip_deserializing)]
    pub direction: Option<String>,
    pub mail: String,
    pub subject: String,
    pub text: String,
}

impl Msg {
    pub fn new(direction: Option<String>, mail: String, subject: String, text: String) -> Self {
        Self {
            direction,
            mail,
            subject,
            text,
        }
    }

    pub fn content_type(&self) -> header::ContentType {
        match Dom::parse(&self.text) {
            Ok(dom) => {
                if dom.children.len() == 1 && dom.children[0].text().is_some() {
                    return header::ContentType::TEXT_PLAIN;
                }

                header::ContentType::TEXT_HTML
            }
            Err(_) => header::ContentType::TEXT_PLAIN,
        }
    }
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            direction: None,
            mail: "my@mail.org".to_string(),
            subject: "My Subject".to_string(),
            text: "My Text".to_string(),
        }
    }
}

pub async fn send(msg: Msg) -> Result<(), ServiceError> {
    let mut message = Message::builder()
        .from(CONFIG.mail.user.parse()?)
        .subject(&msg.subject)
        .header(msg.content_type());

    if msg.direction.is_none() {
        message = message.to(msg.mail.parse()?);
    } else {
        message = message.reply_to(msg.mail.parse()?);

        for recipient in &CONFIG.mail.recipients {
            if Some(&recipient.direction) == msg.direction.as_ref() {
                for mail in &recipient.mails {
                    message = message.to(mail.parse()?);
                }
            }
        }
    }

    if let Ok(mail) = message.body(msg.text) {
        let credentials = Credentials::new(CONFIG.mail.user.clone(), CONFIG.mail.password.clone());

        let transporter = if CONFIG.mail.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&CONFIG.mail.smtp)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&CONFIG.mail.smtp)
        };

        let mailer = transporter?.credentials(credentials).build();

        // Send the mail
        if let Err(e) = mailer.send(mail).await {
            return Err(ServiceError::Conflict(format!("Could not send mail: {e}")));
        }
    } else {
        return Err(ServiceError::InternalServerError);
    }

    Ok(())
}
