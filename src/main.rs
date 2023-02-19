use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use actix_web::{
    post,
    get,
    Error,
    App,
    Responder,
    HttpServer,
    http::header::ContentDisposition,
    error,
    web::{
        Query,
        Json
    }
};
use actix_multipart::Multipart;
use serde::{
    Deserialize,
    Serialize
};
use futures_util::stream::TryStreamExt;
use uuid::Uuid;
use aes_gcm::{
    aead::{
        Aead,
        KeyInit,
        OsRng,
    },
    Aes256Gcm,
    Nonce,
};

// Make this configurable through an environment variable
const FILE_SIZE_LIMIT: usize = 5_000_000;

#[derive(Debug, Serialize)]
struct UploadResponse {
    files: Vec<UploadedFile>,
}

#[derive(Debug, Serialize)]
struct UploadedFile {
    id: Uuid,
    original_file_name: String,
}

#[post("/upload")]
async fn upload(mut payload: Multipart) -> Result<impl Responder, Error> {
    let mut uploaded_files: Vec<UploadedFile> = vec![];

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

        let id = Uuid::new_v4();
        let path = format!("./files/{}", id).to_string();
        let path = Path::new(&path);
        let display = path.display();

        // Time to encrypt the data before storing it on the server
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(b"unique nonce");
        let ciphertext = cipher.encrypt(nonce, bytes.as_ref());

        let mut file = match File::create(&path) {
            Err(reason) => { println!("{:?}", reason); todo!() },
            Ok(file) => file,
        };

        if let Ok(val) = ciphertext { 
            match file.write_all(&val) {
                Err(reason) => { println!("{:?}", reason); todo!() },
                Ok(_) => println!("Successfully wrote to {}", display),
            }
        } else {
            unreachable!("Something stupid happened here, I should learn how to Rust better");
        }

        uploaded_files.push(UploadedFile { id, original_file_name: name.unwrap().to_string() })
    }

    let response: UploadResponse = UploadResponse { files: uploaded_files };

    return Ok(Json(response));
}

#[derive(Debug, Deserialize)]
struct GetFileRequesst {
    id: String,
    // todo: Add authentication token?
}

#[get("/file")]
async fn get_file(params: Query<GetFileRequesst>) -> Result<impl Responder, Error> {
    let path = format!("./files/{}", params.id).to_string();
    let path = Path::new(&path);

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");

    let mut file = File::open(path)?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let buffer = &buffer[..];
    let decrypted = cipher.decrypt(nonce, buffer);

    match decrypted {
        Ok(val) => println!("{:?}", val),
        Err(err) => println!("{:?}", err),
    };
    
    return Ok("OK");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(upload)
            .service(get_file)
    })
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
