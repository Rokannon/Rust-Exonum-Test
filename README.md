# Exonum Timestamping Service

This is a simple project that implements trusted-timestamping service using blockchain technology.  

## Repository structure
```
./exonum - Rust Exonum framework (as a GIT submodule)
./timestamping - Timestamping service Rust-project
./send_transactions - JS-project, that facilitates transactions signing and sending
```

## Project setup
Command line:
- cd to `timestamping` and `run cargo run --package timestamping --example run_service`
- cd to `send_transaction`, run `npm install` and `node index.js`

With IntelliJ IDEA:
- Add repo as an empty project
- Add `timestamping` folder to the project as a Rust module
- Add `send_transaction` folder as HTML module
- Proceed with IDE hint on running `npm install`
- Run `run_service` example to start a demo timestamping service
- Run `index.js` in send_transactions project to add some random 
transactions to the blockchain.

## Service API
```
GET /v1/timestamps - Returns list of all transactions
GET /v1/timestamp/%pub_key% - Returns transaction with %pub_key% key
GET /v1/block_info/%block_height% - Returns info on block with %block_height% height
POST /v1/timestamps - Creates new timestamping transaction
```

To create a new timestamp one must provide a following request body:
```
{
  "body": {
    "pub_key": %public_key%,
    "name": %file_name%
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 1,
  "message_id": 0,
  "signature": %message_signature%
}
```

## Misc.
- Empirical tests show that service supports up to 40 transactions per second. 
- Using same public key twice allows to rewrite structure filename and time values.
- Project `send_transactions` uses `exonum-client` library which makes transaction signing and key-pair generation much easier.

## Conclusion

Exonum framework provides instruments to create a blockchain with custom data structures and methods. Such approach allows to retain data in secure and tamper-proof way. Aforementioned properties are especially useful in hostile data-exchange environment where database clients may not trust each other. The fact that such a system is distributed also provides means for data-redundancy.
