extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate time;

use exonum::blockchain::{Blockchain, Service, Transaction, ApiContext, ExecutionResult,
                         TransactionSet};
use exonum::encoding::serialize::FromHex;
use exonum::node::{TransactionSend, ApiSender};
use exonum::messages::{RawTransaction, Message};
use exonum::storage::{Fork, MapIndex, Snapshot};
use exonum::crypto::{Hash, PublicKey};
use exonum::encoding;
use exonum::api::{Api, ApiError};
use exonum::explorer::BlockchainExplorer;
use iron::prelude::*;
use iron::Handler;
use iron::status::Status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use router::Router;

const SERVICE_ID: u16 = 1;

encoding_struct! {
    struct Timestamp {
        pub_key: &PublicKey,
        name: &str,
        time: u32, // unix-time, seconds
    }
}

pub struct TimestampingSchema<T> {
    view: T,
}

impl<T: AsRef<Snapshot>> TimestampingSchema<T> {
    pub fn new(view: T) -> Self {
        TimestampingSchema { view }
    }

    pub fn timestamps(&self) -> MapIndex<&Snapshot, PublicKey, Timestamp> {
        MapIndex::new("timestamping.timestamps", self.view.as_ref())
    }

    pub fn timestamp(&self, pub_key: &PublicKey) -> Option<Timestamp> {
        self.timestamps().get(pub_key)
    }
}

impl<'a> TimestampingSchema<&'a mut Fork> {
    pub fn timestamps_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Timestamp> {
        MapIndex::new("timestamping.timestamps", &mut self.view)
    }
}

transactions! {
    TimestampingTransactions {
        const SERVICE_ID = SERVICE_ID;

        struct TxCreateTimestamp {
            pub_key: &PublicKey,
            name: &str,
        }
    }
}

impl Transaction for TxCreateTimestamp {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = TimestampingSchema::new(view);
        if schema.timestamp(self.pub_key()).is_none() {
            let timestamp = Timestamp::new(self.pub_key(), self.name(), time::precise_time_s() as u32);
            println!("Create the timestamp: {:?}", timestamp);
            schema.timestamps_mut().put(self.pub_key(), timestamp);
        }
        Ok(())
    }
}

#[derive(Clone)]
struct TimestampingApi {
    channel: ApiSender,
    blockchain: Blockchain,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
}

impl TimestampingApi {
    fn get_timestamp(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let timestamp_key = path.last().unwrap();
        let public_key = PublicKey::from_hex(timestamp_key).map_err(|e| {
            IronError::new(e, (
                Status::BadRequest,
                Header(ContentType::json()),
                "\"Invalid request param: `pub_key`\"",
            ))
        })?;

        let timestamp = {
            let snapshot = self.blockchain.snapshot();
            let schema = TimestampingSchema::new(snapshot);
            schema.timestamp(&public_key)
        };

        if let Some(timestamp) = timestamp {
            self.ok_response(&serde_json::to_value(timestamp).unwrap())
        } else {
            self.not_found_response(&serde_json::to_value("Timestamp not found").unwrap())
        }
    }

    fn get_timestamps(&self, _: &mut Request) -> IronResult<Response> {
        let snapshot = self.blockchain.snapshot();
        let schema = TimestampingSchema::new(snapshot);
        let idx = schema.timestamps();
        let timestamps: Vec<Timestamp> = idx.values().collect();

        self.ok_response(&serde_json::to_value(&timestamps).unwrap())
    }

    fn post_transaction(&self, req: &mut Request) -> IronResult<Response> {
        match req.get::<bodyparser::Struct<TimestampingTransactions>>() {
            Ok(Some(transaction)) => {
                let transaction: Box<Transaction> = transaction.into();
                let tx_hash = transaction.hash();
                self.channel.send(transaction).map_err(ApiError::from)?;
                let json = TransactionResponse { tx_hash };
                self.ok_response(&serde_json::to_value(&json).unwrap())
            }
            Ok(None) => Err(ApiError::BadRequest("Empty request body".into()))?,
            Err(e) => Err(ApiError::BadRequest(e.to_string()))?,
        }
    }

    fn block_info(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let block_height_str = path.last().unwrap();
        let block_height = exonum::helpers::Height(block_height_str.parse::<u64>().unwrap());
        let blockchain_explorer = BlockchainExplorer::new(&self.blockchain);
        match blockchain_explorer.block_info(block_height) {
            Some(block_info) => {
                let result = format!("Block found: {:?}", block_info);
                self.ok_response(&serde_json::to_value(result).unwrap())
            }
            None => {
                self.ok_response(&serde_json::to_value("Block not found!").unwrap())
            }
        }
    }
}

impl Api for TimestampingApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let post_create_timestamp = move |req: &mut Request| self_.post_transaction(req);
        let self_ = self.clone();
        let get_timestamps = move |req: &mut Request| self_.get_timestamps(req);
        let self_ = self.clone();
        let get_timestamp = move |req: &mut Request| self_.get_timestamp(req);
        let self_ = self.clone();
        let block_info = move |req: &mut Request| self_.block_info(req);

        router.post("/v1/timestamps", post_create_timestamp, "post_create_timestamp");
        router.get("/v1/timestamps", get_timestamps, "get_timestamps");
        router.get("/v1/timestamp/:pub_key", get_timestamp, "get_timestamp");
        router.get("/v1/block_info/:block_height", block_info, "block_info");
    }
}

pub struct TimestampingService;

impl Service for TimestampingService {
    fn service_name(&self) -> &'static str {
        "timestamping"
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let tx = TimestampingTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn public_api_handler(&self, ctx: &ApiContext) -> Option<Box<Handler>> {
        let mut router = Router::new();
        let api = TimestampingApi {
            channel: ctx.node_channel().clone(),
            blockchain: ctx.blockchain().clone(),
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }
}
