# Rust Kqueue Challenge — Build Your Own Tiny Event Loop on macOS

## Goal

Build a tiny TCP echo server using:

- Rust
- macOS `kqueue`
- one thread only
- no async/await
- non-blocking sockets

The server should:

- accept multiple clients
- receive text from clients
- echo the text back
- disconnect clients cleanly

---

# Why kqueue Instead of epoll?

`epoll` is Linux-only.

On macOS and BSD systems, the similar native mechanism is:

```text
kqueue
```

You can think of it like:

```text
epoll for macOS/BSD
```

Both let your program say:

```text
Tell me when this socket is ready.
```

---

# Final Mental Model

Your program will eventually look like this:

```text
create listener socket
create kqueue instance
register listener in kqueue

loop forever:
    wait for events

    for each event:
        if listener ready:
            accept client
            register client in kqueue

        if client ready:
            read data
            echo data back
```

---

# Phase 1 — Basic Blocking TCP Server

## Goal

Create a simple blocking TCP echo server.

## Requirements

- listen on `127.0.0.1:9090`
- accept one client
- read bytes
- send same bytes back

## Hints

Use:

```rust
TcpListener
TcpStream
read()
write_all()
```

## Testing

Open another terminal:

```bash
nc 127.0.0.1 9090
```

Type:

```text
hello
```

Server should send:

```text
hello
```

---

# Phase 2 — Non-Blocking Sockets

## Goal

Make sockets non-blocking.

## Requirements

Set:

```rust
listener.set_nonblocking(true)?;
stream.set_nonblocking(true)?;
```

## Important

Now some operations may fail with:

```rust
ErrorKind::WouldBlock
```

This is NOT a real failure.

It means:

```text
No data available right now.
Try later.
```

---

# Phase 3 — Add kqueue

## Goal

Replace blind looping with `kqueue`.

## Suggested Crate

Use the `nix` crate:

```toml
[dependencies]
nix = { version = "0.29", features = ["event"] }
```

## Learn These Concepts

With `kqueue`, you work with:

- `kqueue()` — creates the event queue
- `kevent()` — registers changes and waits for events
- `KEvent` — describes what you want to watch
- `EVFILT_READ` — tells kqueue you care about read readiness

## First Step

Register the listener socket:

```text
listener fd -> EVFILT_READ
```

Meaning:

```text
Wake me when a client is ready to connect.
```

---

# Phase 4 — Track Clients

## Goal

Store connected clients.

## Suggested Structure

```rust
HashMap<RawFd, TcpStream>
```

Key:

```text
file descriptor
```

Value:

```text
client socket
```

---

# Phase 5 — Handle Events

## Listener Event

When listener becomes ready:

```text
accept new client
set client non-blocking
register client fd in kqueue
store client in HashMap
```

Important:

There may be more than one pending client.

So accept in a loop until you get:

```rust
ErrorKind::WouldBlock
```

---

## Client Event

When client becomes ready:

```text
read bytes
```

### If bytes > 0

Echo the same data back.

### If bytes == 0

Client disconnected.

Remove client from:

- kqueue
- HashMap

---

# Tiny Pseudocode

```text
listener = bind 127.0.0.1:9090
listener.set_nonblocking(true)

kq = kqueue()

register listener fd as EVFILT_READ

clients = HashMap<RawFd, TcpStream>

loop:
    events = kevent(kq)

    for event in events:
        fd = event.ident

        if fd == listener_fd:
            loop:
                accept client
                if WouldBlock:
                    break

                client.set_nonblocking(true)
                register client fd as EVFILT_READ
                clients.insert(client_fd, client)

        else:
            client = clients.get_mut(fd)

            read from client

            if bytes_read == 0:
                unregister fd
                clients.remove(fd)

            else:
                write same bytes back
```

---

# Important Concepts to Observe

## 1. kqueue Does Not Hold Data

`kqueue` only says:

```text
This socket is ready.
```

The actual bytes are still inside the kernel socket buffer.

You still need to call:

```rust
read()
```

---

## 2. File Descriptors

Your listener and clients are file descriptors.

```text
listener socket -> fd
client socket   -> fd
```

`kqueue` watches those fds.

---

## 3. Non-Blocking Mode Matters

Without non-blocking mode, your event loop can accidentally freeze.

Example:

```text
one client becomes ready
you read from it
then your read blocks forever
other clients stop being handled
```

Non-blocking mode protects the loop.

---

## 4. Your Loop Owns the Clients

The event loop owns:

```rust
HashMap<RawFd, TcpStream>
```

So when `kqueue` says:

```text
fd 12 is ready
```

You look up:

```rust
clients.get_mut(&12)
```

and handle it.

---

# Suggested Project Structure

```text
kqueue-echo/
 ├── Cargo.toml
 └── src/
     └── main.rs
```

---

# Stretch Goals

## Goal 1 — quit command

If client sends:

```text
quit
```

disconnect only that client.

---

## Goal 2 — Broadcast

If client sends:

```text
broadcast hello
```

send:

```text
hello
```

to all connected clients.

This will force you to think about:

- mutable borrowing
- iterating over clients safely
- ownership during writes

---

## Goal 3 — Simple HTTP Server

Instead of echoing raw text, return an HTTP response:

```text
HTTP/1.1 200 OK
Content-Length: 5

hello
```

Then test it with:

```bash
curl http://127.0.0.1:9090
```

---

# Bonus Learning Questions

While building, try answering:

1. Why does macOS use `kqueue` instead of `epoll`?
2. What does `EVFILT_READ` mean for a listener socket?
3. What does `EVFILT_READ` mean for a client socket?
4. Why do we accept in a loop?
5. Why does `read()` returning `0` mean disconnect?
6. Why is one thread enough for many clients?
7. How would this evolve into a small async runtime?

---

# Final Challenge

After you finish the echo server, build a tiny HTTP server using the same event loop.

That will make `kqueue`, sockets, file descriptors, and event loops feel much more real.
