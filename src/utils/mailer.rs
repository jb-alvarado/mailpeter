use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use html_parser::Dom;
use lettre::{
    message::{
        header::{self, ContentType},
        Attachment, MultiPart, SinglePart,
    },
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};

use crate::utils::errors::ServiceError;
use crate::{ARGS, CONFIG};

#[derive(Debug, Deserialize, Serialize)]
pub struct Msg {
    pub attachment: Option<Vec<(String, Vec<u8>)>>,
    #[serde(skip_deserializing)]
    pub direction: Option<String>,
    pub mail: String,
    pub subject: String,
    pub text: String,
}

impl Msg {
    pub fn new(
        direction: Option<String>,
        mail: String,
        subject: String,
        text: String,
        attachment: Option<Vec<(String, Vec<u8>)>>,
    ) -> Self {
        Self {
            attachment,
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
            attachment: None,
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
        .subject(&msg.subject);

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

    let mut part = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .header(msg.content_type())
            .body(msg.text),
    );

    if let Some(files) = msg.attachment {
        for file in files {
            if let Some(kind) = infer::get(&file.1) {
                let content_type = ContentType::parse(kind.mime_type()).unwrap();
                let attachment = Attachment::new(file.0).body(file.1, content_type);

                part = part.singlepart(attachment);
            } else {
                return Err(ServiceError::Conflict("File type not known!".to_string()));
            };
        }
    }

    if let Ok(mail) = message.multipart(part) {
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

pub async fn cli_sender() -> Result<(), ServiceError> {
    let mut attachment = None;
    let mut recipient = ARGS.recipient.clone();

    if let Some(mut files) = ARGS.attachment.clone() {
        let mut file_collection = vec![];
        let mut size = 0;

        if files.len() > 1 && recipient.is_none() {
            recipient = files.pop();
        }

        for file in files {
            size += fs::metadata(&file)?.len();
            let name = Path::new(&file)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if size > (CONFIG.max_attachment_size_mb * 1048576.0) as u64 {
                return Err(ServiceError::Conflict("Attachment to big!".to_string()));
            }

            file_collection.push((name, fs::read(file)?));
        }

        attachment = Some(file_collection);
    }

    match recipient {
        Some(recipient) => {
            if let Some(text) = &ARGS.text {
                let msg = Msg::new(
                    None,
                    recipient.clone(),
                    ARGS.subject.clone().unwrap(),
                    text.clone(),
                    attachment,
                );

                send(msg).await?;
            } else {
                let stdin = io::stdin();
                let mut stdin_text = vec![];

                for line in stdin.lock().lines() {
                    stdin_text.push(line?);
                }

                let msg = Msg::new(
                    None,
                    recipient.clone(),
                    ARGS.subject.clone().unwrap(),
                    stdin_text.join("\n"),
                    attachment,
                );

                send(msg).await?;
            }
        }
        None => {
            return Err(ServiceError::Conflict(
                "No mail recipient available!".to_string(),
            ));
        }
    }

    Ok(())
}
