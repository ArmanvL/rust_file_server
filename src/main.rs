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
use std::fs::File;
use std::io::Write;
use std::path::Path;

const FILE_SIZE_LIMIT: usize = 5_000_000;

#[post("/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Take the next field from the multipart stream
    while let Some(mut field) = payload.try_next().await? {
        let mut bytes = Vec::<u8>::new();
        {
            while let Some(chunk) = field.try_next().await? {
                bytes.append(&mut chunk.to_vec());

                // Check file size against limit
                if bytes.len() > FILE_SIZE_LIMIT {
                    return Err(error::ErrorBadRequest(format!("File size exceeds file size limit: size - {:?} - limit - {:?}", bytes.len(), FILE_SIZE_LIMIT)));
                }
            }
        }

        if bytes.len() == 0 {
            return Err(error::ErrorBadRequest("Empty file"));
        }

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

        let path = format!("./files/{}", name.unwrap()).to_string();
        let path = Path::new(&path);
        let display = path.display();
        
        let mut file = match File::create(&path) {
            Err(reason) => { println!("{:?}", reason); todo!() },
            Ok(file) => file,
        };

        match file.write_all(&bytes) {
            Err(reason) => { println!("{:?}", reason); todo!() },
            Ok(_) => println!("Successfully wrote to {}", display),
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
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
