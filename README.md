# tcp-kqueue

> Build a **tiny TCP echo server** on **macOS** with **`kqueue`**: **one thread**, **non-blocking sockets**, **no async/await**.

📄 **Full spec:** [`task.md`](task.md) — phases, pseudocode, concepts, bonus questions.

> 💡 On GitHub, use the **Outline** sidebar (or `⊞` / table-of-contents control) to jump between sections.

---

## 🎯 Goal

By the end of the challenge, the server should:

| | |
|---|--|
| ✅ | Accept **multiple** clients |
| ✅ | **Echo** the same bytes you receive |
| ✅ | **Disconnect** cleanly when a client goes away |

Rough control flow:

```text
listener + kqueue → register fds → loop: kevent → accept / read / write / teardown
```

---

## 📦 Prerequisites

| Requirement | Notes |
|-------------|--------|
| **Rust** | This crate uses **Edition 2024**; use a current stable toolchain. |
| **macOS** (recommended) | **`kqueue`** is the BSD/macOS counterpart to Linux **`epoll`**. Linux won’t run the native `kqueue` path. |

---

## 🗺️ Phases at a glance

> Exact bind addresses, `nc` / `curl` commands, and hints are in [**`task.md`**](task.md).

| Phase | Topic |
| :---: | --- |
| **1** | Blocking TCP — `TcpListener`, `TcpStream`, `read()`, `write_all()` |
| **2** | **Non-blocking** sockets — `ErrorKind::WouldBlock` is normal |
| **3** | **`kqueue`** — e.g. `nix` + `event`: `kqueue()`, `kevent()`, `EVFILT_READ` |
| **4** | **Track clients** — e.g. `HashMap<RawFd, TcpStream>` |
| **5** | **Event loop** — accept until `WouldBlock`, read/echo, remove on EOF |

<details>
<summary><strong>Example <code>nix</code> dependency</strong> (when you reach Phase 3)</summary>

```toml
[dependencies]
nix = { version = "0.29", features = ["event"] }
```

</details>

---

## 🛠️ Running & testing

```bash
cargo build
cargo run
```

- **Bind address, port, and manual checks** (`nc`, `curl`, etc.) — see the matching section in [`task.md`](task.md).
- **`task.md`** is the source of truth for requirements and how to verify behavior.

---

## 🎁 Stretch goals

| Goal | What to try |
|------|-------------|
| **`quit`** | Drop only that client |
| **`broadcast …`** | Fan-out a message to every connection |
| **Mini HTTP** | Same event loop, return a small `HTTP/1.1` response |

---

## 📁 Project layout

```text
tcp-kqueue/
├── Cargo.toml
├── README.md          ← overview (this file)
├── task.md            ← detailed challenge
└── src/
    └── main.rs
```

---

## 🤔 Why kqueue?

```text
Linux  → epoll
macOS / BSD → kqueue
```

Both answer: *“Which fds are ready right now?”* — different kernels, same event-driven idea.

---

## 📚 See also

| Doc | What’s inside |
|-----|----------------|
| [`task.md`](task.md) | Phase requirements, pseudocode, FD/kqueue mental model, learning questions |
