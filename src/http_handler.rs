use std::io::prelude::*;

use http::header::{HeaderName, HeaderValue};
use http::method::Method;
use http::request::Builder;
use http::uri::Uri;
use http::version::Version;
use http::{HeaderMap, Request, Response};
use std::str;

pub fn response_to_string<T>(response: Response<T>) -> Result<String, Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let response = response;
    let mut buffer = Vec::new();
    write!(buffer, "HTTP/1.1 {}\r\n", response.status().as_str())?;
    for (name, value) in response.headers() {
        write!(buffer, "{}: {}\r\n", name, value.to_str()?)?;
    }
    write!(buffer, "\r\n")?;
    let body = serde_json::to_string(response.body())?;
    write!(buffer, "{}", body)?;
    Ok(String::from_utf8(buffer)?)
}

pub fn write_to_stream<T>(
    stream: &mut std::net::TcpStream,
    response: Response<T>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    stream.write(
        response_to_string(response)
            .expect("RESPONSE_SERIALIZER: Error while serializing the response to send.")
            .as_bytes(),
    )?;
    stream.flush()?;
    Ok(())
}

pub fn read_from_stream<T>(
    stream: &mut std::net::TcpStream,
) -> Result<Request<T>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned + Default,
{
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    Ok(parse_request(&buffer)?)
}

fn parse_request_line(
    request_line: &str,
) -> Result<(Method, Uri, Version), Box<dyn std::error::Error>> {
    let mut request_line_parts = request_line.split_whitespace();
    let method = request_line_parts.next().unwrap_or_default().parse::<Method>()?;
    let uri = request_line_parts.next().unwrap_or_default().parse::<Uri>()?;
    let version = match request_line_parts.next().unwrap_or_default() {
        "HTTP/1.1" => Version::HTTP_11,
        "HTTP/1.0" => Version::HTTP_10,
        _ => panic!("Unsupported HTTP version"),
    };
    Ok((method, uri, version))
}

fn parse_headers(headers: &mut HeaderMap, lines: &mut str::Split<&str>) -> Result<(), Box<dyn std::error::Error>>  {
    for line in lines {
        if line.is_empty() {
            break;
        }
        let mut header_parts = line.splitn(2, ": ");
        let name = header_parts.next().unwrap_or_default().parse::<HeaderName>()?;
        let value = header_parts.next().unwrap_or_default().parse::<HeaderValue>()?;
        headers.append(name, value);
    }
    Ok(())
}

fn parse_body<T>(lines: &mut str::Split<&str>) -> T
where
    T: serde::de::DeserializeOwned + Default,
{
    serde_json::from_str(
        lines
            .next()
            .unwrap_or_default()
            .trim_matches(|c: char| c.is_whitespace() || c.is_control()),
    )
    .unwrap_or(T::default())
}

fn parse_request<T>(buffer: &[u8]) -> Result<Request<T>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned + Default,
{
    let request_str = str::from_utf8(buffer)?;
    let mut lines = request_str.split("\r\n");

    // Parse the request line
    let request_line = parse_request_line(lines.next().unwrap_or_default())?;
    let method = request_line.0;
    let uri = request_line.1;
    let version = request_line.2;

    // Parse headers
    let mut headers = http::HeaderMap::new();
    parse_headers(&mut headers, &mut lines)?;

    // Parse the body
    let body: T = if method == Method::POST && lines.clone().count() > 0 {
        parse_body(&mut lines)
    } else {
        T::default()
    };

    // Build the request struct
    let mut request = Builder::new()
        .method(method)
        .uri(uri)
        .version(version)
        .body(body)?;
    request.headers_mut().extend(headers);
    Ok(request)
}
