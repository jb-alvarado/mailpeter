use actix_web::{post, web, Responder};
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
