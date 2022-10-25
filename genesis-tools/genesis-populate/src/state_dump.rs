use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives::types::StateRoot;
use near_store::db::TestDB;
use near_store::{DBCol, Store, Temperature};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const STATE_DUMP_FILE: &str = "state_dump";
const GENESIS_ROOTS_FILE: &str = "genesis_roots";

pub struct StateDump {
    pub store: Store,
    pub roots: Vec<StateRoot>,
}

impl StateDump {
    pub fn from_dir(dir: &Path, store_home_dir: &Path, in_memory_db: bool) -> Self {
        let node_storage = if in_memory_db {
            let storage = TestDB::new();
            near_store::NodeStorage::new(storage)
        } else {
            near_store::NodeStorage::opener(store_home_dir, &Default::default(), None)
                .open()
                .unwrap()
        };
        let store = node_storage.get_store(Temperature::Hot);
        let state_file = dir.join(STATE_DUMP_FILE);
        store
            .load_from_file(DBCol::State, state_file.as_path())
            .expect("Failed to read state dump");
        let roots_files = dir.join(GENESIS_ROOTS_FILE);
        let mut file = File::open(roots_files).expect("Failed to open genesis roots file.");
        let mut data = vec![];
        file.read_to_end(&mut data).expect("Failed to read genesis roots file.");
        let roots: Vec<StateRoot> =
            BorshDeserialize::try_from_slice(&data).expect("Failed to deserialize genesis roots");
        Self { store, roots }
    }

    pub fn save_to_dir(self, dir: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut dump_path = dir.clone();
        dump_path.push(STATE_DUMP_FILE);
        self.store.save_to_file(DBCol::State, dump_path.as_path())?;
        {
            let mut roots_files = dir;
            roots_files.push(GENESIS_ROOTS_FILE);
            let mut file = File::create(roots_files)?;
            let data = self.roots.try_to_vec()?;
            file.write_all(&data)?;
        }
        Ok(())
    }
}
