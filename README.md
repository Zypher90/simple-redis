# 🧠 Rust Redis — A Concurrent In-Memory Database

A high-performance, concurrent in-memory key-value store built in Rust, inspired by systems like Redis. This project explores backend system design, async programming, and concurrency patterns using Tokio.

---

## 🚀 Overview

This project implements a TCP-based key-value database that supports multiple concurrent clients, a custom command protocol, append-only persistence, and a message-passing architecture to efficiently handle requests without shared-state locking.

It was built to deeply understand:

* async I/O in Rust
* concurrency models (shared state vs message passing)
* protocol design over TCP
* durability via logging

---

## ✨ Features

* ⚡ Async TCP server using Tokio
* 🔌 Multiple concurrent client connections
* 🧩 Custom command protocol (`SET`, `GET`, `DELETE`, `UPDATE`)
* 🧠 Message-passing architecture (no direct shared-state locking)
* 💾 Append-only log persistence (AOF-style)
* 🔄 Automatic recovery on restart
* 🧪 Modular, testable design

---

## 🏗️ Architecture

```
Client
  ↓
Connection Task (async per client)
  ↓
Command Parsing
  ↓
Channel (mpsc)
  ↓
Worker Task (owns store)
  ↓
Execution Engine
  ↓
Persistence Layer (append-only log)
  ↓
Response → back to client
```

### Key Design Decisions

* **Message Passing over Locks**
  Initially used shared state (`Mutex`), later refactored to a worker-based model using channels to eliminate contention and simplify ownership.

* **Append-Only Persistence**
  All write operations are logged to disk. On startup, the log is replayed to rebuild the in-memory state.

* **Separation of Concerns**
  Clear boundaries between networking, parsing, execution, and storage for maintainability and extensibility.

---

## 📡 Protocol

Simple, line-based text protocol over TCP:

### Commands

```
SET <key> <value>
GET <key>
DELETE <key>
UPDATE <key> <value>
```

### Responses

```
OK
VALUE <data>
NIL
ERR <message>
```

---

## 🧪 Example Usage

### Start the server

```bash
cargo run --bin server
```

### Start an instance of test client

```bash
cargo run --bin test-client
```

### Run commands

```
SET name Alice
GET name
DELETE name
UPDATE name Joe
```

### Output

```
OK
VALUE Alice
OK
NIL
```

---

## 💾 Persistence

* Write operations (`SET`, `DELETE`) are appended to a log file
* On server startup:

  * log file is read line-by-line
  * commands are replayed to rebuild state

### Trade-offs

* ✔ Simple and reliable
* ✔ Easy recovery model
* ✖ Log grows indefinitely (no compaction yet)

---

---

## 🧠 What I Learned

* Designing concurrent systems in Rust using async/await
* Trade-offs between shared-state and message-passing architectures
* Handling real-world TCP stream issues (partial reads, buffering)
* Building durable systems with logging and recovery
* Structuring backend systems with clean separation of concerns

---

## 🚧 Future Improvements

* Key expiration (TTL support)
* Log compaction / snapshotting
* Replication between nodes
* Binary protocol support
* Metrics and observability

---

## 🏁 Why This Project

Most backend projects focus on REST APIs. This project goes deeper—into:

* networking
* concurrency
* systems design
* data durability

It demonstrates the ability to build **low-level, high-performance backend systems in Rust**.

---

## 📜 License

MIT
