use actix_multipart::Multipart;
use actix_web::{post, put, web, Responder};
use futures_util::TryStreamExt as _;
use log::error;

use crate::utils::{
    errors::ServiceError,
    mailer::{send, Msg},
};

#[post("/mail/{direction}/")]
pub async fn post_mail(
    direction: web::Path<String>,
    mut data: web::Json<Msg>,
) -> Result<impl Responder, ServiceError> {
    data.direction = Some(direction.into_inner());

    match send(data.into_inner()).await {
        Ok(_) => Ok("Send success!"),
        Err(e) => {
            error!("{e:?}");
            Err(ServiceError::InternalServerError)
        }
    }
}

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
        mail,
        subject,
        text,
        Some(files),
    );

    match send(msg).await {
        Ok(_) => Ok("Send success!"),
        Err(e) => {
            error!("{e:?}");
            Err(ServiceError::InternalServerError)
        }
    }
}
