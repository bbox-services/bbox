use crate::fastcgi_h::*;
use log::{debug, error, info};
use std;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::mem;
use std::os::unix::net::UnixStream;

fn connect_socket() -> UnixStream {
    UnixStream::connect("/tmp/asyncfcgi").unwrap()
}

pub fn fcgi_request(query_string: &str) {
    let mut stream = connect_socket();
    write_request(&mut stream, query_string).unwrap();
    read_response(&mut stream)
}

fn write_header<W: Write>(
    w: &mut W,
    record_type: u8,
    request_id: u16,
    content_len: usize,
) -> io::Result<()> {
    assert!(content_len <= std::u32::MAX as usize);
    let request_id = unsafe { mem::transmute::<_, [u8; 2]>(request_id.to_be()) };
    let content_length = unsafe { mem::transmute::<_, [u8; 2]>((content_len as u16).to_be()) };
    w.write_all(&[
        FCGI_VERSION_1,
        record_type,
        request_id[0],
        request_id[1],
        content_length[0],
        content_length[1],
        0,
        0,
    ])?;
    Ok(())
}

fn write_len<W: Write>(w: &mut W, n: u32) -> io::Result<()> {
    if n < 0x80 {
        w.write_all(&[n as u8])?;
    } else {
        assert!(n < 0x80000000);
        let buf = unsafe { mem::transmute::<u32, [u8; 4]>((0x80000000 | n).to_be()) };
        w.write_all(&buf)?;
    }
    Ok(())
}

fn write_pairs<W: Write>(w: &mut W, pairs: Vec<(&str, &str)>) -> io::Result<()> {
    for (key, value) in pairs {
        write_len(w, key.len() as u32)?;
        write_len(w, value.len() as u32)?;
        write!(w, "{}{}", key, value)?;
    }
    Ok(())
}

fn write_params<W: Write>(w: &mut W, request_id: u16, query_string: &str) -> io::Result<()> {
    let params = vec![
        ("REQUEST_METHOD", "GET"),
        ("SERVER_NAME", "localhost"),
        ("REQUEST_URI", "/"),
        ("QUERY_STRING", query_string),
    ];
    let mut content_len = 0;
    for (k, v) in &params {
        content_len += if k.len() < 0x80 { 1 } else { 4 };
        content_len += if v.len() < 0x80 { 1 } else { 4 };
        content_len += k.len() + v.len();
    }

    write_header(w, FCGI_PARAMS, request_id, content_len)?;
    write_pairs(w, params)?;
    Ok(())
}

fn write_request(stream: &mut UnixStream, query_string: &str) -> io::Result<()> {
    // Set an arbitrary non-null FCGI RequestId
    let request_id = 1;

    // Send FCGI_BEGIN_REQUEST
    write_header(stream, FCGI_BEGIN_REQUEST, request_id, 8)?;
    // FCGI_BeginRequestBody
    stream.write(&[
        0,
        FCGI_RESPONDER,
        0, /* keep_conn: false */
        0,
        0,
        0,
        0,
        0,
    ])?;

    // Send environment to the FCGI application
    write_params(stream, request_id, query_string)?;

    // Send EOF
    write_header(stream, FCGI_STDIN, request_id, 0)?;

    Ok(())
}

fn read_response(stream: &mut UnixStream) {
    let mut body = Vec::<u8>::new();
    let mut reader = BufReader::new(stream);
    loop {
        let mut header = [0; 8];
        reader.read(&mut header).unwrap();
        let header = unsafe { mem::transmute::<_, FCGI_Header>(header) };
        let content_length = unsafe {
            u16::from_be(mem::transmute([
                header.contentLengthB1,
                header.contentLengthB0,
            ])) as usize
        };
        match header.type_ {
            FCGI_STDOUT => {
                if content_length > 0 {
                    read_stdout_response(&mut reader, content_length, &mut body);
                }
                read_padding(&mut reader, header.paddingLength as usize);
            }
            FCGI_STDERR => {
                let mut fcgi_stderr = String::with_capacity(content_length);
                reader.read_to_string(&mut fcgi_stderr).unwrap();
                info!("fcgi_stderr: {}", fcgi_stderr);
            }
            FCGI_END_REQUEST => {
                let mut end_request_body = [0; 8];
                reader.read(&mut end_request_body).unwrap();
                //debug!("FCGI_END_REQUEST: {:?}", end_request_body);
                // response = resp.body(body);
                break;
            }
            _ => {
                error!("Unexpected FCGI response header: {:?}", header);
                break;
            }
        }
    }
}

fn read_stdout_response(
    reader: &mut BufReader<&mut UnixStream>,
    content_length: usize,
    body: &mut Vec<u8>,
) {
    let mut body_length = content_length;

    if body.len() == 0 {
        // Read header lines
        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line).unwrap();
            body_length -= len;
            if len <= 1 {
                break;
            }
            let kv: Vec<&str> = line.split(": ").collect();
            debug!("Body header: {:?}", kv);
            if kv.len() == 2 {
                let _val = &kv[1][0..kv[1].len() - 1];
                match kv[0] {
                    "Content-Type" => {
                        // resp.header(kv[0], val);
                    }
                    "Status" => {
                        // resp.status(
                        //     StatusCode::from_u16(u16::from_str(&val[1..]).unwrap()).unwrap(),
                        // );
                    }
                    _ => {}
                }
            }
        }
    }

    // Read body content
    reader.take(body_length as u64).read_to_end(body).unwrap();
}

fn read_padding(reader: &mut BufReader<&mut UnixStream>, padding_length: usize) {
    let mut padding = Vec::with_capacity(padding_length);
    reader
        .take(padding_length as u64)
        .read_to_end(&mut padding)
        .unwrap();
}
