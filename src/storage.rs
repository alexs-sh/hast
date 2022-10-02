use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Info {
    pub id: String,
    pub host: Option<String>,
    pub timestamp: Option<String>,
}

impl Info {
    pub fn new(id: &str) -> Info {
        Info {
            id: id.to_owned(),
            host: None,
            timestamp: None,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectHash {
    pub name: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InsertRequest {
    pub info: Info,
    pub records: Vec<ObjectHash>,
}

impl InsertRequest {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub struct InsertResonse {}

#[derive(Serialize, Deserialize)]
pub struct LookupRequest {
    pub hashes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LookupResponse {
    pub records: Vec<Info>,
}
impl LookupResponse {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub trait Storage: Send + Sync {
    fn insert(&mut self, data: InsertRequest) -> Option<InsertResonse>;
    fn lookup(&self, data: LookupRequest) -> Option<LookupResponse>;
}

pub struct EmptyStorage;

impl Storage for EmptyStorage {
    fn insert(&mut self, _: InsertRequest) -> Option<InsertResonse> {
        None
    }
    fn lookup(&self, _: LookupRequest) -> Option<LookupResponse> {
        None
    }
}

pub struct SimpleStorage {
    records: HashMap<String, InsertRequest>, //info -> set of hash
    hashes: HashMap<String, HashSet<String>>, // hash -> set of records
    workdir: String,
}

impl Storage for SimpleStorage {
    fn insert(&mut self, data: InsertRequest) -> Option<InsertResonse> {
        if self.save_mem(&data) {
            self.save_file(&data).ok()?;
        }
        Some(InsertResonse {})
    }

    fn lookup(&self, data: LookupRequest) -> Option<LookupResponse> {
        let mut infos = HashMap::new();
        info!("requested {} hash(es)", data.hashes.len());
        for hash in data.hashes {
            if let Some(records) = self.hashes.get(&hash) {
                for id in records {
                    let info = self.records.get(id).unwrap();
                    infos.insert(id, info.info.clone());
                }
            }
        }
        let records = infos.iter().fold(Vec::new(), |mut acc, (_, v)| {
            acc.push(v.clone());
            acc
        });

        info!("found {} record(s)", records.len());
        if records.is_empty() {
            None
        } else {
            Some(LookupResponse { records })
        }
    }
}

impl SimpleStorage {
    pub fn new(workdir: &str) -> SimpleStorage {
        SimpleStorage {
            records: HashMap::new(),
            hashes: HashMap::new(),
            workdir: workdir.to_owned(),
        }
    }

    pub fn init(self) -> io::Result<SimpleStorage> {
        std::fs::create_dir_all(&self.workdir)?;
        self.load()
    }

    fn load(mut self) -> io::Result<SimpleStorage> {
        let read_file = |path: &Path| -> io::Result<InsertRequest> {
            let mut buffer = String::new();

            let mut file = File::open(path).map_err(|err| {
                warn!("can't open file {:?}:{}", path, err);
                err
            })?;

            file.read_to_string(&mut buffer).map_err(|err| {
                warn!("can't open file {:?}:{}", path, err);
                err
            })?;

            let data: InsertRequest = serde_json::from_str(&buffer).map_err(|err| {
                warn!("can't parse JSON from file {:?}:{}", path, err);
                err
            })?;

            Ok(data)
        };

        let paths = std::fs::read_dir(self.workdir.clone())?;
        for path in paths {
            let _ = path.map(|p| {
                read_file(p.path().as_path()).map(|data| {
                    self.insert(data);
                })
            });
        }
        Ok(self)
    }

    fn save_file(&mut self, data: &InsertRequest) -> io::Result<()> {
        let hash = {
            let mut s = DefaultHasher::new();
            data.info.id.hash(&mut s);
            s.finish()
        };

        let name = hash.to_string();

        info!(
            "saving '{}' with {} record(s) to file",
            data.info.id,
            data.records.len()
        );

        let data = data.to_json();
        let path = Path::new(&self.workdir).join(name.clone());

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(|err| {
                error!("can't open file {}:{}", name, err);
                err
            })?;

        file.write_all(data.as_bytes()).map_err(|err| {
            error!("can't write file {}:{}", name, err);
            err
        })
    }

    fn save_mem(&mut self, data: &InsertRequest) -> bool {
        // make hash->records links
        data.records.iter().for_each(|rec| {
            let hash = &rec.hash;
            if let Some(records) = self.hashes.get_mut(hash) {
                records.insert(data.info.id.clone());
            } else {
                self.hashes
                    .insert(hash.clone(), HashSet::from_iter(vec![data.info.id.clone()]));
            }
        });

        // fill records info
        let mut result = false;
        self.records
            .entry(data.info.id.clone())
            .and_modify(|e| {
                warn!("record '{}' exists. Skip it", e.info.id);
            })
            .or_insert_with(|| {
                info!("insert record '{}'", data.info.id);
                result = true;
                data.clone()
            });
        result
    }
}
