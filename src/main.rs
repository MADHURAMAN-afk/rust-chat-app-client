use std::io::{self, ErrorKind, Read, Write}; // Importing tools for input/output operations.
use std::net::TcpStream; // Importing tools for network communication.
use std::sync::mpsc::{self, TryRecvError}; // Importing tools for multi-threaded communication.
use std::{thread, vec}; // Importing threading and vector tools.
use std::time::Duration; // Importing tools to handle time.

const LOCAL: &str = "127.0.0.1:8090"; // Defining the address we'll connect to.
const MSG_SIZE: usize = 32; // Defining the maximum size of messages.

fn main() {


    let mut clients = TcpStream::connect(LOCAL).expect("stream failed to connect"); // Connecting to the server.
    clients.set_nonblocking(true).expect("failed to initiate non-blocking"); // Setting the connection to non-blocking mode.

    let (tx, rx) = mpsc::channel::<String>(); // Creating a channel for sending and receiving messages between threads.
    thread::spawn(move || loop { // Spawning a new thread to handle communication.
        let mut buff = vec![0; MSG_SIZE]; // Creating a buffer to store incoming messages.
        match clients.read_exact(&mut buff) { // Reading a message from the server.
            Ok(_) => { // If a message is successfully received.
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>(); // Converting the message into readable format.
                println!("message recv {:?}", msg); // Printing the received message.
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // If no message is received, continue.
            Err(_) => { // If an error occurs while reading.
                println!("connection with server was severed"); // Print a message indicating connection was lost.
                break; // Exit the loop.
            }
        }

        match rx.try_recv() { // Trying to receive a message from the main thread.
            Ok(msg) => { // If a message is received.
                let mut buff = msg.clone().into_bytes(); // Converting the message into bytes.
                buff.resize(MSG_SIZE, 0); // Resizing the message to fit the buffer.
                clients.write_all(&buff).expect("writing to socket failed"); // Sending the message to the server.
                println!("message sent {:?}", msg); // Printing the sent message.
            }
            Err(TryRecvError::Empty) => (), // If no message is received, continue.
            Err(TryRecvError::Disconnected) => break, // If the channel is disconnected, exit the loop.
        }

        thread::sleep(Duration::from_millis(100)); // Sleeping for a short time to prevent busy-waiting.
    });

    println!("write a message:"); // Prompting the user to write a message.
    loop {
        let mut buff = String::new(); // Creating a buffer to store user input.
        io::stdin().read_line(&mut buff).expect("reading from stdin failed"); // Reading user input from stdin.
        let msg = buff.trim().to_string(); // Trimming and converting the input into a string.

        if msg == ":quit" || tx.send(msg).is_err() { // If the user enters ":quit" or there's an error sending the message.
            break; // Exit the loop.
        }
    }

    println!("bye bye!"); // Print a farewell message.
}
