use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind("127.0.0.1:9090")?;

    println!("Server is listening on 127.0.0.1:9090");

    let (mut stream, addr) = listner.accept()?;
    let mut buffer = [0; 1024];

    println!("Connection established with {}", addr);
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n])?;
    }

    Ok(())
}
