use actix_web::{HttpResponse, web};

#[derive(serde::Deserialize)]
pub struct FormData {
   pub email: String,
   pub name: String,
}

pub async fn subscribe(_form_data: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().into()
}
