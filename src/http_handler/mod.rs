pub mod error;

use std::io::prelude::*;

use error::NinoverseHttpHandlerError;
use http::header::{HeaderName, HeaderValue};
use http::method::Method;
use http::request::Builder;
use http::uri::Uri;
use http::version::Version;
use http::{HeaderMap, Request, Response};
use std::str;

pub fn response_to_string<T>(response: Response<T>) -> Result<String, NinoverseHttpHandlerError>
where
    T: serde::Serialize,
{
    let response = response;
    let mut buffer = Vec::new();
    write!(buffer, "HTTP/1.1 {}\r\n", response.status().as_str()).or_else(|_| {
        Err(NinoverseHttpHandlerError::BufferError {
            additional_info: String::from("Writing status to buffer."),
        })
    })?;
    for (name, value) in response.headers() {
        let header_value = value.to_str().or_else(|_| {
            Err(NinoverseHttpHandlerError::BufferError {
                additional_info: String::from("Converting header value to &str."),
            })
        })?;
        write!(buffer, "{}: {}\r\n", name, header_value).or_else(|_| {
            Err(NinoverseHttpHandlerError::BufferError {
                additional_info: String::from("Writing header name and value."),
            })
        })?;
    }
    write!(buffer, "\r\n").or_else(|_| {
        Err(NinoverseHttpHandlerError::BufferError {
            additional_info: String::from("Writing basic characters to buffer."),
        })
    })?;
    let body = serde_json::to_string(response.body()).or_else(|_| {
        Err(NinoverseHttpHandlerError::BufferError {
            additional_info: String::from("Serializing body with serde_json."),
        })
    })?;
    write!(buffer, "{}", body).or_else(|_| {
        Err(NinoverseHttpHandlerError::BufferError {
            additional_info: String::from("Writing body to buffer."),
        })
    })?;
    Ok(String::from_utf8(buffer).or_else(|_| {
        Err(NinoverseHttpHandlerError::BufferError {
            additional_info: String::from("Converting buffer to String."),
        })
    })?)
}

pub fn write_to_stream<T>(
    stream: &mut std::net::TcpStream,
    response: Response<T>,
) -> Result<(), NinoverseHttpHandlerError>
where
    T: serde::Serialize,
{
    stream
        .write(response_to_string(response)?.as_bytes())
        .or_else(|_| {
            Err(NinoverseHttpHandlerError::StreamError {
                additional_info: String::from("Writing to stream."),
            })
        })?;
    stream.flush().or_else(|_| {
        Err(NinoverseHttpHandlerError::StreamError {
            additional_info: String::from("Flushing stream."),
        })
    })?;
    Ok(())
}

pub fn read_from_stream<T>(
    stream: &mut std::net::TcpStream,
) -> Result<Request<T>, NinoverseHttpHandlerError>
where
    T: serde::de::DeserializeOwned + Default,
{
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).or_else(|_| {
        Err(NinoverseHttpHandlerError::StreamError {
            additional_info: String::from("Reading stream."),
        })
    })?;
    Ok(parse_request(&buffer)?)
}

fn parse_request_line(
    request_line: &str,
) -> Result<(Method, Uri, Version), NinoverseHttpHandlerError> {
    let mut request_line_parts = request_line.split_whitespace();
    let method = request_line_parts
        .next()
        .unwrap_or_default()
        .parse::<Method>()
        .or_else(|_| {
            Err(NinoverseHttpHandlerError::ParsingError {
                additional_info: String::from("Parsing response method."),
            })
        })?;
    let uri = request_line_parts
        .next()
        .unwrap_or_default()
        .parse::<Uri>()
        .or_else(|_| {
            Err(NinoverseHttpHandlerError::ParsingError {
                additional_info: String::from("Parsing URI."),
            })
        })?;
    let version = match request_line_parts.next().unwrap_or_default() {
        "HTTP/1.1" => Ok(Version::HTTP_11),
        "HTTP/1.0" => Ok(Version::HTTP_10),
        _ => Err(NinoverseHttpHandlerError::StreamError {
            additional_info: String::from("Reading stream."),
        }),
    }?;
    Ok((method, uri, version))
}

fn parse_headers(
    headers: &mut HeaderMap,
    lines: &mut str::Split<&str>,
) -> Result<(), NinoverseHttpHandlerError> {
    for line in lines {
        if line.is_empty() {
            break;
        }
        let mut header_parts = line.splitn(2, ": ");
        let name = header_parts
            .next()
            .unwrap_or_default()
            .parse::<HeaderName>()
            .or_else(|_| {
                Err(NinoverseHttpHandlerError::ParsingError {
                    additional_info: String::from("Parsing header name."),
                })
            })?;
        let value = header_parts
            .next()
            .unwrap_or_default()
            .parse::<HeaderValue>()
            .or_else(|_| {
                Err(NinoverseHttpHandlerError::ParsingError {
                    additional_info: String::from("Parsing header value."),
                })
            })?;
        headers.append(name, value);
    }
    Ok(())
}

fn parse_body<T>(lines: &mut str::Split<&str>) -> Result<T, NinoverseHttpHandlerError>
where
    T: serde::de::DeserializeOwned + Default,
{
    Ok(serde_json::from_str(
        lines
            .next()
            .unwrap_or_default()
            .trim_matches(|c: char| c.is_whitespace() || c.is_control()),
    )
    .or_else(|_| {
        Err(NinoverseHttpHandlerError::ParsingError {
            additional_info: String::from("Parsing body value."),
        })
    })?)
}

fn parse_request<T>(buffer: &[u8]) -> Result<Request<T>, NinoverseHttpHandlerError>
where
    T: serde::de::DeserializeOwned + Default,
{
    let request_str = str::from_utf8(buffer).or_else(|_| {
        Err(NinoverseHttpHandlerError::ParsingError {
            additional_info: String::from("Parsing URI."),
        })
    })?;
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
        parse_body(&mut lines)?
    } else {
        T::default()
    };

    // Build the request struct
    let mut request = Builder::new()
        .method(method)
        .uri(uri)
        .version(version)
        .body(body)
        .or_else(|_| {
            Err(NinoverseHttpHandlerError::RequestStructError {
                additional_info: String::from("Building request struct."),
            })
        })?;
    request.headers_mut().extend(headers);
    Ok(request)
}

pub fn extract_uri_pieces_vector<T>(request: &Request<T>) -> Vec<String> {
    let uri_pieces = request
        .uri()
        .path()
        .trim_start_matches('/')
        .split('/')
        .map(|uri_part| String::from(uri_part))
        .collect::<Vec<String>>();
    uri_pieces
}
