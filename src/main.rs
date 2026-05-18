use nix::{
    sys::event::{EvFlags, EventFilter, FilterFlag, KEvent, Kqueue},
};
use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

fn blank_event() -> KEvent {
    KEvent::new(
        0,
        EventFilter::EVFILT_READ,
        EvFlags::empty(),
        FilterFlag::empty(),
        0,
        0,
    )
}

fn subscribe_read(fd: RawFd) -> KEvent {
    KEvent::new(
        fd as usize,
        EventFilter::EVFILT_READ,
        EvFlags::EV_ADD | EvFlags::EV_ENABLE,
        FilterFlag::empty(),
        0,
        0,
    )
}

fn unsubscribe_read(fd: RawFd) -> KEvent {
    KEvent::new(
        fd as usize,
        EventFilter::EVFILT_READ,
        EvFlags::EV_DELETE,
        FilterFlag::empty(),
        0,
        0,
    )
}

fn handle_new_connection(
    listener: &TcpListener,
    clients: &mut HashMap<RawFd, TcpStream>,
    kernel_updates: &mut Vec<KEvent>,
) -> std::io::Result<()> {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true)?;
                let fd = stream.as_raw_fd();
                println!("New client connected: fd={fd}");
                clients.insert(fd, stream);
                kernel_updates.push(subscribe_read(fd));
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn handle_client(
    fd: RawFd,
    clients: &mut HashMap<RawFd, TcpStream>,
    kernel_updates: &mut Vec<KEvent>,
) {
    let mut buf = [0; 1024];
    let count = match clients.get_mut(&fd) {
        Some(stream) => match stream.read(&mut buf) {
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::WouldBlock => return,
            Err(e) => {
                eprintln!("read error on fd {fd}: {e}");
                kernel_updates.push(unsubscribe_read(fd));
                clients.remove(&fd);
                return;
            }
        },
        None => return,
    };

    if count == 0 {
        println!("Client fd={fd} disconnected");
        kernel_updates.push(unsubscribe_read(fd));
        clients.remove(&fd);
        return;
    };

    let Some(stream) = clients.get_mut(&fd) else {
        return;
    };

    let mut sent = 0;
    while sent < count {
        match stream.write(&buf[sent..count]) {
            Ok(n) => sent += n,
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(e) => {
                eprintln!("write error on fd {fd}: {e}");
                kernel_updates.push(unsubscribe_read(fd));
                clients.remove(&fd);
                return;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9090")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let kqueue = Kqueue::new()?;
    // Register first event to the kernel before next wait
    let mut kernel_updates = vec![subscribe_read(listener_fd)];
    let mut clients: HashMap<RawFd, TcpStream> = HashMap::new();
    let mut events = [blank_event(); 32];

    println!("Server is running on 127.0.0.1:9090");

    loop {
        // block until kernel notifies us of an event
        let n = kqueue.kevent(&kernel_updates, &mut events, None)?;
        // clear updates list as we have applied them
        kernel_updates.clear();

        for event in events.iter().take(n) {
            let fd = event.ident() as RawFd;
            if fd == listener_fd {
                match handle_new_connection(&listener, &mut clients, &mut kernel_updates) {
                    Ok(()) => (),
                    Err(e) => eprintln!("Error accepting new connection: {e}"),
                }
            } else {
                handle_client(fd, &mut clients, &mut kernel_updates);
            }
        }
    }
}
