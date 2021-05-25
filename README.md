# grpc-address-book

[![build](https://github.com/mosmeh/grpc-address-book/workflows/build/badge.svg)](https://github.com/mosmeh/grpc-address-book/actions)

Address book service over gRPC

## Server

```bash
cp .env.sample .env
vi .env
sqlx database setup
cargo run --bin server
```

## Client

```bash
cargo run --bin client -- list
cargo run --bin client -- create --name "David Davis" --email "david@example.com"
cargo run --bin client -- search --name "david"
```
