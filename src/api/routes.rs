use actix_multipart::Multipart;
use actix_web::{post, put, web, Responder};
use futures_util::TryStreamExt as _;
use log::{error, trace};

use crate::utils::{
    errors::ServiceError,
    mailer::{message_worker, Msg},
};

/// This Rust code handles HTTP POST and PUT requests related to sending emails.

/// The **post_mail** function is an asynchronous function that handles POST requests to the
/// "/mail/{direction}/" endpoint. The **{direction}** in the URL is a path parameter, which
/// is captured and passed to the function as the **direction** argument. The function also
/// accepts a JSON payload in the request body, which is deserialized into a **Msg** struct.
/// The **direction** is then added to the **Msg** struct. The **send** function is called
/// with the **Msg** struct to send the email. If the email is sent successfully, the function
/// returns a success message.If there is an error, it logs the error and returns an
/// **InternalServerError** response.
#[post("/mail/{direction}/")]
pub async fn post_mail(
    direction: web::Path<String>,
    mut msg: web::Json<Msg>,
) -> Result<impl Responder, ServiceError> {
    msg.direction = Some(direction.into_inner());

    trace!("Msg: {:?}", msg.clone());

    if msg.is_spam() {
        return Err(ServiceError::UnprocessableEntity(
            "Message contains blocked words".to_string(),
        ));
    }

    match message_worker(msg.into_inner()).await {
        Ok(_) => Ok("Send success!"),
        Err(_) => Err(ServiceError::InternalServerError),
    }
}

/// The **put_mail_attachment** function is another asynchronous function that handles PUT requests
/// to the "/mail/{direction}/" endpoint. This function is designed to handle multipart form
/// data, which is commonly used for file uploads.  The function initializes empty vectors and
/// strings to hold the files and form fields from the request. It then enters a loop where it
/// tries to get the next field from the multipart form data.  If the field has a
/// **content_disposition** of "mail", "subject", or "text", the function reads the next chunk of
/// data from the field and converts it to a string. If the field has a different
/// **content_disposition**, the function assumes it's a file and reads the file data into a
/// buffer. The filename and buffer are then added to the **files** vector. If the field has a
/// content_disposition that the function doesn't recognize, it logs an error and returns a
/// **Conflict** response.
#[put("/mail/{direction}/")]
pub async fn put_mail_attachment(
    direction: web::Path<String>,
    mut payload: Multipart,
) -> Result<impl Responder, ServiceError> {
    let mut files = vec![];
    let mut mail = String::new();
    let mut subject = String::new();
    let mut text = String::new();

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition().clone();
        if let Some(name) = content_disposition.get_name() {
            match name {
                "mail" => {
                    if let Some(chunk) = field.try_next().await? {
                        mail = String::from_utf8_lossy(&chunk).to_string();
                    }
                }
                "subject" => {
                    if let Some(chunk) = field.try_next().await? {
                        subject = String::from_utf8_lossy(&chunk).to_string();
                    }
                }
                "text" => {
                    if let Some(chunk) = field.try_next().await? {
                        text = String::from_utf8_lossy(&chunk).to_string();
                    }
                }
                _ => {
                    if let Some(filename) = content_disposition.get_filename() {
                        let mut buffer: Vec<u8> = vec![];

                        while let Some(chunk) = field.try_next().await? {
                            for slices in chunk.iter().copied() {
                                buffer.push(slices);
                            }
                        }

                        files.push((filename.to_string(), buffer));
                    } else {
                        error!("Unknown form data: {name}");
                        return Err(ServiceError::Conflict(format!("Unknown form data: {name}")));
                    }
                }
            }
        }
    }

    let msg = Msg::new(
        Some(direction.into_inner()),
        false,
        mail,
        subject,
        text,
        Some(files),
    );

    trace!("Msg: {msg:?}");

    if msg.is_spam() {
        return Err(ServiceError::UnprocessableEntity(
            "Message contains blocked words".to_string(),
        ));
    }

    match message_worker(msg).await {
        Ok(_) => Ok("Send success!"),
        Err(_) => Err(ServiceError::InternalServerError),
    }
}
