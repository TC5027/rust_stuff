use rand::Rng;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;

const TCPLISTENER_KEY: u64 = 777;

const HTTP_RESP: &[u8] = b"HTTP/1.1 200 OK
content-type: text/html
content-length: 5

Hello";

enum Action {
    Reading,
    Writing,
}

fn main() -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let mut map_key_stream: HashMap<u64, TcpStream> = HashMap::new();
    let mut map_key_action: HashMap<u64, Action> = HashMap::new();

    // We create a file descriptor associated to a TcpListener
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    listener.set_nonblocking(true)?;
    let listener_file_descriptor = listener.as_raw_fd();

    // We create an epoll instance
    let epoll_file_descriptor = unsafe { libc::epoll_create1(0) };

    // We add listener_file_descriptor to the interest list of our epoll instance
    let mut epoll_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: TCPLISTENER_KEY,
    };
    unsafe {
        libc::epoll_ctl(
            epoll_file_descriptor,
            libc::EPOLL_CTL_ADD,
            listener_file_descriptor,
            &mut epoll_event,
        )
    };

    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    loop {
        // We loop on epoll_wait
        let number_ready =
            unsafe { libc::epoll_wait(epoll_file_descriptor, events.as_mut_ptr(), 1024, 1000) };
        //println!("Got {} events", number_ready);
        unsafe {
            events.set_len(number_ready as usize);
        }
        // We have number_ready file descriptors ready for I/O
        for event in &events {
            match event.u64 {
                // The TcpListener is ready for I/O meaning there is a new incoming connection
                TCPLISTENER_KEY => {
                    //println!("The TcpListener got something");
                    match listener.accept() {
                        Ok((stream, _address)) => {
                            stream.set_nonblocking(true)?;
                            let stream_file_descriptor = stream.as_raw_fd();
                            let key = rng.gen::<u64>();
                            // We remember the mapping between the generated key and this TcpStream
                            map_key_stream.insert(key, stream);
                            map_key_action.insert(key, Action::Reading);
                            // We add the file descriptor for this stream to the interest list of our epoll instance
                            let mut epoll_event = libc::epoll_event {
                                events: libc::EPOLLIN as u32,
                                u64: key,
                            };
                            unsafe {
                                libc::epoll_ctl(
                                    epoll_file_descriptor,
                                    libc::EPOLL_CTL_ADD,
                                    stream_file_descriptor,
                                    &mut epoll_event,
                                )
                            };
                        }
                        Err(_) => {
                            // TODO
                        }
                    }
                }
                // A TcpStream is ready for I/O
                key => {
                    match map_key_action.get(&key).unwrap() {
                        Action::Reading => {
                            // note :  here we can see the level trigger behaviour as if
                            // we set buf small enough, for example [0u8; 256] so that one 
                            // request is too big to be read with a single call to read, we have, 
                            // with only 1 connexion at the server's adress, epoll_wait which is awaken
                            // automatically 2 times for this stream (which corresponds to the number
                            // of times we had to read to have the whole content
                            // of the request in my case)
                            //println!("A TcpStream got something");
                            // We get the TcpStream associated to this key
                            let mut stream = map_key_stream.get(&key).unwrap();
                            // We read its content
                            let mut buf = [0u8; 256];
                            stream.read(&mut buf)?;
                            let content = std::str::from_utf8(&buf).unwrap();
                            //println!("content read : {:?}", content);
                            // we check if we are done reading (dumb! do not reproduce)
                            if content.contains("\r\n\r\n") {
                                // we change the event from reading to writing for this TcpStream
                                let mut epoll_event = libc::epoll_event {
                                    events: libc::EPOLLOUT as u32,
                                    u64: key,
                                };
                                unsafe {
                                    libc::epoll_ctl(
                                        epoll_file_descriptor,
                                        libc::EPOLL_CTL_MOD,
                                        stream.as_raw_fd(),
                                        &mut epoll_event,
                                    )
                                };
                                map_key_action.insert(key, Action::Writing);
                            }
                        }
                        Action::Writing => {
                            // We get the TcpStream associated to this key
                            let mut stream = map_key_stream.get(&key).unwrap();
                            // We write the HTTP response
                            stream.write(HTTP_RESP).unwrap();
                            // We close the stream and remove it from the epoll instance
                            stream.shutdown(std::net::Shutdown::Both).unwrap();
                            let stream_file_descriptor = stream.as_raw_fd();
                            unsafe {
                                libc::epoll_ctl(
                                    epoll_file_descriptor,
                                    libc::EPOLL_CTL_DEL,
                                    stream_file_descriptor,
                                    std::ptr::null_mut(),
                                )
                            };
                            // We close the file descriptor as there is a limit
                            // for the number of file descriptors one process can open (1024)
                            unsafe { libc::close(stream_file_descriptor) };
                        }
                    }
                }
            }
        }
    }
}
