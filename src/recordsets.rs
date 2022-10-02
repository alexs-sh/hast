use crate::storage::{Info, ObjectHash};
use log::{info, warn};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
pub type Id = u64;

fn append<T>(storage: &mut HashMap<Id, HashSet<T>>, key: Id, value: T)
where
    T: Eq + Hash + Copy,
{
    storage
        .entry(key)
        .and_modify(|set| {
            set.insert(value);
        })
        .or_insert_with(|| {
            let mut set = HashSet::new();
            set.insert(value);
            set
        });
}

fn id_from_str(value: &str) -> Id {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

type IdStorage = HashMap<Id, String>;

pub struct RecordSet {
    // Full data info
    reports: HashMap<Id, Info>,

    // Names -> Ids
    name_ids: IdStorage, // storage for names <-> ids
    hash_ids: IdStorage, // storage for hash <-> ids

    // Relations
    hash_to_name: HashMap<Id, HashSet<Id>>, // hash (id) -> file (id)
    hash_to_report: HashMap<Id, HashSet<Id>>, // hash (id) -> report (id)
}

impl RecordSet {
    pub fn new() -> RecordSet {
        RecordSet {
            reports: HashMap::new(),

            hash_ids: IdStorage::new(),
            name_ids: IdStorage::new(),

            hash_to_name: HashMap::new(),
            hash_to_report: HashMap::new(),
        }
    }

    pub fn insert(&mut self, info: &Info, objects: &[ObjectHash]) -> bool {
        let mut changed = false;
        let report_id = id_from_str(&info.id); // make record id

        self.reports
            .entry(report_id)
            .and_modify(|_| {
                warn!("skip report '{}'. Already in a storage", info.id);
            })
            .or_insert_with(|| {
                info!(
                    "insert report '{}' with {} record(s)",
                    info.id,
                    objects.len()
                );
                // write all name/hash ids
                for object in objects {
                    let name_id = id_from_str(&object.object);
                    let hash_id = id_from_str(&object.hash);

                    self.name_ids
                        .entry(name_id)
                        .or_insert_with(|| object.object.clone());
                    self.hash_ids
                        .entry(hash_id)
                        .or_insert_with(|| object.hash.clone());

                    append(&mut self.hash_to_report, hash_id, report_id);
                    append(&mut self.hash_to_name, hash_id, name_id);
                }
                changed = true;
                info.clone()
            });

        info!(
            "number of (reports,hashes,names):({},{},{})",
            self.reports.len(),
            self.hash_ids.len(),
            self.name_ids.len()
        );
        changed
    }

    pub fn get_by_hash(&self, name: &str) -> Option<(Vec<&Info>, Vec<&String>)> {
        let hash_id = id_from_str(name); // record id
        let infos = self.hash_to_report.get(&hash_id).map(|set| {
            let mut v = Vec::new();
            for rid in set {
                v.push(self.reports.get(rid).unwrap());
            }
            v
        });

        let names = self.hash_to_name.get(&hash_id).map(|set| {
            let mut v = Vec::new();
            for nid in set {
                v.push(self.name_ids.get(nid).unwrap());
            }
            v
        });

        match (infos, names) {
            (Some(info), Some(name)) => Some((info, name)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reports() {
        let mut records = RecordSet::new();
        let info1 = Info::new("report1");
        let objects1 = ObjectHash {
            object: "report1-file".to_owned(),
            hash: "1234".to_owned(),
        };

        let info2 = Info::new("report2");
        let objects2 = ObjectHash {
            object: "report2-file".to_owned(),
            hash: "2345".to_owned(),
        };

        records.insert(&info1, &[objects1]);
        records.insert(&info2, &[objects2]);

        let (infos, names) = records.get_by_hash("1234").unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "report1");

        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "report1-file");

        let (infos, names) = records.get_by_hash("2345").unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "report2");

        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "report2-file");

        assert_eq!(records.get_by_hash("none"), None);
    }
}
