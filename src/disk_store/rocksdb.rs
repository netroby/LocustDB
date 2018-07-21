extern crate rocksdb;

use std::sync::Arc;

use bincode::{serialize, deserialize};
use byteorder::{ByteOrder, LittleEndian};
use self::rocksdb::*;

use disk_store::interface::*;
use mem_store::column::Column;

struct RocksDB {
    db: DB,
}

impl RocksDB {
    pub fn new(path: &str) -> RocksDB {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open_cf(&options, path, &vec!["metadata", "partitions"]).unwrap();
        RocksDB {
            db,
        }
    }

    fn metadata(&self) -> ColumnFamily {
        self.db.cf_handle("metadata").unwrap()
    }

    fn partitions(&self) -> ColumnFamily {
        self.db.cf_handle("partitions").unwrap()
    }
}

impl DiskStore for RocksDB {
    fn load_metadata(&self) -> Vec<PartitionMetadata> {
        let mut metadata = Vec::new();
        let iter = self.db.iterator_cf(self.metadata(), IteratorMode::Start).unwrap();
        for (key, value) in iter {
            let partition_id = LittleEndian::read_u64(&key) as PartitionID;
            let MetaData { tablename, columns } = deserialize(&value).unwrap();
            metadata.push(PartitionMetadata {
                id: partition_id,
                tablename,
                columns,
            })
        }
        metadata
    }

    fn load_column_data(&self, partition: PartitionID, column_name: &str) -> Vec<u8> {
        panic!("")
    }

    fn store_partition(&self, partition: PartitionID, tablename: &str, columns: &Vec<Arc<Column>>) {
        let mut tx = WriteBatch::default();

        let mut key = [0; 8];
        LittleEndian::write_u64(&mut key, partition as u64);
        let md = MetaData {
            tablename: tablename.to_string(),
            columns: columns.iter().map(|c| c.name().to_string()).collect(),
        };
        tx.put_cf(self.metadata(), &key, &serialize(&md).unwrap());

        for column in columns {
            let mut key = Vec::new();
            LittleEndian::write_u64(&mut key, partition as u64);
            key.push('.' as u8);
            key.extend(column.name().as_bytes());
            // TODO(clemens): serialise column data
            let data = Vec::new();
            tx.put_cf(self.partitions(), &key, &data);
        }

        self.db.write(tx);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MetaData {
    tablename: String,
    columns: Vec<String>,
}
