use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // listen TCP connection

    // iter through all open connection
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established");

        handle_connection_v2(stream);
    }
}

fn handle_connection_v2(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let req_status_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, filename) = if req_status_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 400 NOT FOUND", "404.html")
    };

    let content = fs::read_to_string(filename).unwrap();
    let content_length = content.len();
    let resp = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{content}");

    stream.write(resp.as_bytes()).unwrap();
}

pub fn handle_connection_v1(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let req_status_line = buf_reader.lines().next().unwrap().unwrap();
    println!("{}", req_status_line);

    if req_status_line == "GET / HTTP/1.1" {
        // let http_request: Vec<_> = buf_reader
        //     .lines() // split by newline
        //     .map(|result| result.unwrap()) // read data from result
        //     .take_while(|line| !line.is_empty()) // end when there is no more content
        //     .collect(); // to vector

        // /* Request format
        // Method Request-URI HTTP-Version CRLF
        // headers CRLF
        // message-body
        // */
        // println!("request {:#?}", http_request);

        /*Response format
        HTTP-Version Status-Code Reason-Phrase CRLF <- status line
        headers CRLF
        message-body
        */

        let status_line = "HTTP/1.1 200 OK";
        let content = fs::read_to_string("hello.html").unwrap(); // read html file
        let length = content.len();

        // status_line -> headers -> content
        let resp = format!("{status_line}\r\nContent-Length: {length}\r\n\n\n{content}");

        stream.write(resp.as_bytes()).unwrap();
    } else {
        let status_line = "HTTP/1.1 400 NOT FOUND";
        let content = fs::read_to_string("404.html").unwrap();
        let length = content.len();

        let resp = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

        stream.write(resp.as_bytes()).unwrap();
    }
}
