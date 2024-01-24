use std::{
    ffi::OsStr,
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
    AsyncFileTransport, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use log::{error, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use voca_rs::Voca;

use crate::utils::errors::ServiceError;
use crate::{ARGS, CONFIG};

/// Ignore lines from stdin, that starts with this strings
const IGNORE_LINES: [&str; 9] = [
    "From:",
    "To:",
    "Subject:",
    "MIME-Version:",
    "Content-Type:",
    "Content-Transfer-Encoding:",
    "X-Cron-Env:",
    "X-Cron-Env:",
    "X-Cron-Env:",
];

/// Mail struct
///
/// This struct contains the mail data, that is send to the mail server.
/// The struct is used for serialization and deserialization.
///
/// The struct has the following fields:
/// * **attachment** - A vector of tuples, that contains the file name and the file content
/// * **direction** - A string that contains the mail direction
/// * **mail** - A string that contains the mail address
/// * **subject** - A string that contains the mail subject
/// * **text** - A string that contains the mail text
///
/// The struct has the following methods:
/// * **new** - The constructor for the struct
/// * **content_type** - Returns the content type of the mail text
/// * **default** - Returns the default struct
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Msg {
    pub attachment: Option<Vec<(String, Vec<u8>)>>,
    #[serde(skip_deserializing)]
    pub direction: Option<String>,
    #[serde(skip_deserializing)]
    pub allow_html: bool,
    pub mail: String,
    pub subject: String,
    pub text: String,
}

/// The `Msg` struct has an associated `new` function, which is a constructor that takes values for
/// each of the struct's fields and returns a new `Msg` object.
///
/// The `content_type` method returns the content type of the email message. It tries to parse the
/// `text` field as a `Dom` object (presumably representing a Document Object Model). If the parsing
/// is successful and the `Dom` object has exactly one child that contains text, it returns `TEXT_PLAIN`
/// as the content type. Otherwise, it returns `TEXT_HTML`. If the parsing fails, it also returns `TEXT_PLAIN`.
///
/// The `is_spam` method returns true if the `text` or `subject` fields contain any of the words in the
/// `block_words` field of the `CONFIG` object.
impl Msg {
    pub fn new(
        direction: Option<String>,
        allow_html: bool,
        mail: String,
        subject: String,
        text: String,
        attachment: Option<Vec<(String, Vec<u8>)>>,
    ) -> Self {
        Self {
            attachment,
            allow_html,
            direction,
            mail,
            subject,
            text,
        }
    }

    pub fn content_type(&self) -> header::ContentType {
        match Dom::parse(&self.text) {
            Ok(dom) => {
                if (dom.children.len() == 1 && dom.children[0].text().is_some()) || !self.allow_html
                {
                    return header::ContentType::TEXT_PLAIN;
                }

                header::ContentType::TEXT_HTML
            }
            Err(_) => header::ContentType::TEXT_PLAIN,
        }
    }

    pub fn is_spam(&self) -> bool {
        let mut spam = false;

        for word in &CONFIG.mail.block_words {
            let re = Regex::new(&format!(r"\b{}\b", word)).unwrap();
            if re.is_match(&self.subject) || re.is_match(&self.text) {
                spam = true;
                break;
            }
        }

        spam
    }
}

/// The `Msg` struct also implements the `Default` trait, which provides a `default` method that returns a
/// `Msg` object with default values for each field.
impl Default for Msg {
    fn default() -> Self {
        Self {
            attachment: None,
            allow_html: false,
            direction: None,
            mail: "my@mail.org".to_string(),
            subject: "My Subject".to_string(),
            text: "My Text".to_string(),
        }
    }
}

/// Take Msg object and send it to the mail server
pub async fn send(mut msg: Msg) -> Result<(), ServiceError> {
    let mut message = Message::builder().subject(&msg.subject);

    // full_name comes mostly from system mails and it is implemented to be compatible with sendmail
    match &ARGS.full_name {
        Some(full_name) => {
            message = message.from(format!("{full_name} <{}>", CONFIG.mail.user).parse()?)
        }
        None => message = message.from(CONFIG.mail.user.parse()?),
    };

    // directions are used to send mails to different recipients and comes from API routes
    if msg.direction.is_none() {
        message = message.to(msg.mail.parse()?);
    } else {
        message = message.reply_to(msg.mail.parse()?);

        for recipient in &CONFIG.mail.recipients {
            if Some(&recipient.direction) == msg.direction.as_ref() {
                msg.allow_html = recipient.allow_html;

                for mail in &recipient.mails {
                    message = message.to(mail.parse()?);
                }
            }
        }
    }

    let message_text = if msg.allow_html {
        msg.text.clone()
    } else {
        msg.text._strip_tags()
    };

    // create multipart mail to support attachments
    let mut part = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .header(msg.content_type())
            .body(message_text),
    );

    // add attachments to mail if available
    if let Some(files) = msg.attachment {
        for file in files {
            let mime_type = match infer::get(&file.1) {
                Some(kind) => kind.mime_type(),
                // application/octet-stream is the default mime type
                None => "application/octet-stream",
            };

            let content_type = ContentType::parse(mime_type).unwrap();
            let attachment = Attachment::new(file.0).body(file.1, content_type);

            part = part.singlepart(attachment);
        }
    }

    let mail = message.multipart(part)?;
    let credentials = Credentials::new(CONFIG.mail.user.clone(), CONFIG.mail.password.clone());

    // create transporter based on starttls configuration
    let transporter = if CONFIG.mail.starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&CONFIG.mail.smtp)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&CONFIG.mail.smtp)
    };

    let mailer = transporter?.credentials(credentials).build();

    trace!("Mail: {mail:?}");

    mailer.send(mail.clone()).await?;

    // backup mail to file if mail_archive is set
    if !CONFIG.mail_archive.is_empty() && Path::new(&CONFIG.mail_archive).is_dir() {
        let backup = AsyncFileTransport::<Tokio1Executor>::new(Path::new(&CONFIG.mail_archive));

        backup.send(mail).await?;
    }

    Ok(())
}

/// Send mail from command line arguments
pub async fn cli_sender() -> Result<(), ServiceError> {
    let mut attachment = None;
    let mut recipient = ARGS.recipient.clone();

    if let Some(mut files) = ARGS.attachment.clone() {
        let mut file_collection = vec![];
        let mut size = 0;

        if files.len() > 1 && recipient.is_none() {
            recipient = files.pop();
        }

        // read files from disk and add them to attachment
        for file in files {
            size += fs::metadata(&file)?.len();
            let name = Path::new(&file)
                .file_name()
                .unwrap_or(OsStr::new("file"))
                .to_string_lossy()
                .to_string();

            // check if file is to big
            if size > (CONFIG.max_attachment_size_mb * 1048576.0) as u64 {
                error!(
                    "Attachment to big! {size} > {max}",
                    size = size,
                    max = CONFIG.max_attachment_size_mb * 1048576.0
                );

                return Err(ServiceError::Conflict("Attachment to big!".to_string()));
            }

            file_collection.push((name, fs::read(file)?));
        }

        attachment = Some(file_collection);
    }

    // set subject if available
    let mut subject = match ARGS.subject.clone() {
        Some(s) => s,
        None => "No Subject".to_string(),
    };

    match recipient {
        Some(mut recipient) => {
            if !recipient.contains('@') && !CONFIG.mail.alias.is_empty() {
                recipient = CONFIG.mail.alias.clone();
            }

            // set text if available or read from stdin
            if let Some(text) = &ARGS.text {
                let msg = Msg::new(
                    None,
                    true,
                    recipient.clone(),
                    subject,
                    text.clone(),
                    attachment,
                );

                send(msg).await?;
            } else {
                let stdin = io::stdin();
                let mut stdin_text = vec![];

                // read stdin line by line and ignore lines that starts with IGNORE_LINES
                // extract subject and recipient from stdin
                for line in stdin.lock().lines() {
                    let line = line?;
                    if line.starts_with("Subject:") {
                        subject = line.split_once(": ").unwrap().1.to_string();
                    } else if line.starts_with("To:") {
                        let res = line.split_once(": ").unwrap().1.to_string();
                        if res.contains('@') {
                            recipient = res;
                        }
                    } else if !(ignore_start(&line) || stdin_text.is_empty() && line.is_empty()) {
                        stdin_text.push(line);
                    }
                }

                let msg = Msg::new(
                    None,
                    true,
                    recipient.clone(),
                    subject,
                    stdin_text.join("\n"),
                    attachment,
                );

                trace!("Msg: {msg:?}");

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

/// Check if line starts with IGNORE_LINES
fn ignore_start(input: &str) -> bool {
    for ignore_line in IGNORE_LINES.iter() {
        if input.starts_with(ignore_line) {
            return true;
        }
    }

    false
}
