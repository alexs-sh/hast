use crate::storage::{Info, InsertRequest, LookupRequest, LookupResponse, ObjectHash, Storage};
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind};
use std::sync::RwLock;

pub struct Config {
    pub address: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            address: "0.0.0.0:8888".to_owned(),
        }
    }
    pub fn with_address(mut self, address: &str) -> Self {
        self.address = address.to_owned();
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct Payload {
    data: String,
}

#[derive(Deserialize)]
struct InsertJson {
    info: Info,
    payload: Payload,
}

fn make_error(kind: ErrorKind, txt: &str) -> Error {
    error!("{}", txt);
    Error::new(kind, txt)
}

fn make_error_silent(kind: ErrorKind) -> Error {
    Error::new(kind, "")
}

impl InsertJson {
    fn read_payload(&self) -> io::Result<Vec<ObjectHash>> {
        let decoded = base64::decode(self.payload.data.as_str())
            .map_err(|_| make_error(ErrorKind::InvalidData, "can't decode base64 data"))?;

        let result = String::from_utf8(decoded)
            .map_err(|_| make_error(ErrorKind::InvalidData, "can't decode UTF-8 string"))?
            .lines()
            .fold(Vec::new(), |mut acc: Vec<ObjectHash>, line| {
                if let Some(delim) = line.find(' ') {
                    let hash = line[..delim].trim();
                    let object = line[delim..].trim();
                    let record = ObjectHash {
                        object: object.to_owned(),
                        hash: hash.to_owned(),
                    };
                    acc.push(record);
                }
                acc
            });

        info!("request '{}' has {} records", self.info.id, result.len());

        Ok(result)
    }
}

fn make_status(ok: bool) -> String {
    match ok {
        true => "{\"status\":\"ok\"}",
        _ => "{\"status\":\"failed\"}",
    }
    .to_owned()
}

struct SharedData {
    storage: Box<dyn Storage>,
}

type Context = Data<RwLock<SharedData>>;

async fn lookup(json: web::Json<LookupRequest>, ctx: Context) -> String {
    info!("request {} hash(es)", json.hashes.len());
    let run_request = || -> io::Result<LookupResponse> {
        let locked = ctx
            .write()
            .map_err(|_| make_error(ErrorKind::Other, "DB is blocked"))?;
        match locked.storage.lookup(json.0) {
            Some(response) => Ok(response),
            None => Err(make_error_silent(ErrorKind::Other)),
        }
    };
    match run_request() {
        Ok(response) => response.to_json(),
        Err(_) => make_status(false),
    }
}

async fn insert(json: web::Json<InsertJson>, ctx: Context) -> String {
    info!("request '{}' received", json.info.id);
    info!(
        "request '{}' has payload of {} bytes",
        json.info.id,
        json.payload.data.len()
    );

    let run_request = || -> io::Result<()> {
        let records = json.read_payload()?;
        let mut locked = ctx
            .write()
            .map_err(|_| make_error(ErrorKind::Other, "DB is blocked"))?;
        let request = InsertRequest {
            info: json.info.clone(),
            records,
        };
        match locked.storage.insert(request) {
            Some(_) => Ok(()),
            None => Err(make_error(ErrorKind::Other, "DB error")),
        }
    };

    let result = run_request();
    make_status(result.is_ok())
}

pub async fn run(config: Config, storage: Box<dyn Storage>) -> io::Result<()> {
    let shared = Data::new(RwLock::new(SharedData { storage }));
    HttpServer::new(move || {
        let json_cfg = web::JsonConfig::default().limit(32 * 1024 * 1024);
        App::new()
            .app_data(json_cfg)
            .app_data(shared.clone())
            .route("/insert", web::post().to(insert))
            .route("/lookup", web::get().to(lookup))
    })
    .bind(config.address)?
    .run()
    .await?;
    Ok(())
}
