use std::io::Bytes;

use actix_web::{
    post,
    Error,
    App,
    HttpResponse,
    HttpServer,
    http::header::ContentDisposition, error,
};
use actix_multipart::Multipart;
use futures_util::stream::TryStreamExt;

const FILE_SIZE_LIMIT: usize = 1_000_000;

#[post("/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Take the next field from the multipart stream
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition: &ContentDisposition = field.content_disposition();
        let name: Option<&str> = content_disposition.get_filename();

        match name {
            Some(val) => {
                if val == "" {
                    return Err(error::ErrorBadRequest("Empty/no file name"));
                }
            },
            None => return Err(error::ErrorBadRequest("Empty/no file name")),
        }

        let mut bytes = Vec::<u8>::new();

        while let Some(chunk) = field.try_next().await? {
            bytes.append(&mut chunk.to_vec());

            // Check file size against limit
            if bytes.len() > FILE_SIZE_LIMIT {
                return Err(error::ErrorBadRequest("File size exceeds file size limit"));
            }
        }

        if bytes.len() == 0 {
            return Err(error::ErrorBadRequest("Empty file"));
        }
    }
    return Ok(HttpResponse::Ok().body("Yes"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(upload)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
