use std::{
    io::{ErrorKind, Read, Write},
    net::TcpListener,
};

fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind("127.0.0.1:9090")?;

    println!("Server is listening on 127.0.0.1:9090");

    listner.set_nonblocking(true)?;

    loop {
        match listner.accept() {
            Ok((mut stream, _addr)) => loop {
                stream.set_nonblocking(true)?;

                let mut buf = [0; 1024];

                match stream.read(&mut buf) {
                    Ok(0) => break, // client disconnected
                    Ok(n) => {
                        stream.write_all(&buf[..n])?;
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        // no data yet. keep going
                    }
                    Err(e) => eprintln!("Accepting connection error: {e}"),
                }
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                // no pending connections
            }
            Err(e) => eprintln!("Accepting connection error: {e}"),
        }
    }
}
