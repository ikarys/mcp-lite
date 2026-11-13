#![cfg(feature = "oauth")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use mcp_lite::oauth::{McpCredentials, begin, finish, refresh};
use mcp_lite::{ClientInfo, McpServer, McpTransport};

struct TestServer {
    base: String,
    requests: Arc<Mutex<Vec<(String, String, String)>>>,
}

impl TestServer {
    fn requests(&self) -> Vec<(String, String, String)> {
        self.requests.lock().unwrap().clone()
    }
}

fn start_server<F>(route: F) -> TestServer
where
    F: Fn(&str, &str, &str) -> (u16, String, Vec<(String, String)>) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let base = format!("http://127.0.0.1:{port}");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let server_requests = requests.clone();
    let thread_base = base.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            let Ok((method, path, body)) = read_request(&mut stream) else {
                continue;
            };
            server_requests
                .lock()
                .unwrap()
                .push((method.clone(), path.clone(), body.clone()));
            let (status, response_body, extra_headers) = route(&method, &path, &thread_base);
            let _ = write_response(&mut stream, status, &response_body, &extra_headers);
        }
    });

    TestServer { base, requests }
}

fn read_request(stream: &mut impl Read) -> Result<(String, String, String), std::io::Error> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while !buf.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte)? == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection closed",
            ));
        }
        buf.push(byte[0]);
    }

    let header_end = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap()
        + 4;
    let header_text = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let mut lines = header_text.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut content_length = 0usize;
    for line in lines {
        if let Some((key, value)) = line.split_once(':')
            && key.trim().eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = buf[header_end..].to_vec();
    while body.len() < content_length {
        if stream.read(&mut byte)? == 0 {
            break;
        }
        body.push(byte[0]);
    }
    body.truncate(content_length);

    Ok((method, path, String::from_utf8_lossy(&body).into_owned()))
}

fn write_response(
    stream: &mut impl Write,
    status: u16,
    body: &str,
    extra_headers: &[(String, String)],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "OK",
    };

    let mut response = String::new();
    response.push_str("HTTP/1.1 ");
    response.push_str(&status.to_string());
    response.push(' ');
    response.push_str(reason);
    response.push_str("\r\ncontent-type: application/json\r\n");
    response.push_str(&format!("content-length: {}\r\n", body.len()));
    response.push_str("connection: close\r\n");
    for (name, value) in extra_headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    response.push_str(body);

    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn http_server(name: &str, url: &str) -> McpServer {
    McpServer {
        name: name.into(),
        transport: McpTransport::Http,
        url: Some(url.to_string()),
        command: None,
        ..Default::default()
    }
}

fn client_info() -> ClientInfo<'static> {
    ClientInfo {
        name: "test",
        version: "0",
    }
}

#[test]
fn the_full_flow_registers_authorizes_and_exchanges_the_code() {
    let server = start_server(|method, path, base| match (method, path) {
        ("POST", "/mcp") => (
            401,
            "{}".to_string(),
            vec![(
                "WWW-Authenticate".to_string(),
                format!("Bearer resource_metadata=\"{base}/.well-known/oauth-protected-resource\""),
            )],
        ),
        ("GET", "/.well-known/oauth-protected-resource") => (
            200,
            format!(r#"{{"resource":"{base}/mcp","authorization_servers":["{base}"]}}"#),
            vec![],
        ),
        ("GET", "/.well-known/oauth-authorization-server") => (
            200,
            format!(
                r#"{{"authorization_endpoint":"{base}/authorize","token_endpoint":"{base}/token","registration_endpoint":"{base}/register"}}"#
            ),
            vec![],
        ),
        ("POST", "/register") => (
            200,
            r#"{"client_id":"registered-client"}"#.to_string(),
            vec![],
        ),
        ("POST", "/token") => (
            200,
            r#"{"access_token":"access-token","refresh_token":"refresh-token","expires_in":3600}"#
                .to_string(),
            vec![],
        ),
        _ => (404, "{}".to_string(), vec![]),
    });

    let mcp_server = http_server("mcp", &format!("{}/mcp", server.base));
    let authorization = begin(&mcp_server, "http://127.0.0.1:1/callback", &client_info()).unwrap();

    assert!(
        authorization
            .url
            .starts_with(&format!("{}/authorize", server.base))
    );
    assert!(authorization.url.contains("response_type=code"));
    assert!(authorization.url.contains("client_id=registered-client"));
    assert!(authorization.url.contains("code_challenge_method=S256"));
    assert!(
        authorization
            .url
            .contains(&format!("state={}", authorization.state))
    );
    assert!(
        authorization
            .url
            .contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A1%2Fcallback")
    );
    assert!(authorization.url.contains("resource=http%3A%2F%2F"));

    let credentials = finish(authorization.pending, "the-code").unwrap();
    assert_eq!(credentials.access_token, "access-token");
    assert_eq!(credentials.refresh_token.as_deref(), Some("refresh-token"));
    assert_eq!(credentials.client_id, "registered-client");
    assert_eq!(credentials.token_endpoint, format!("{}/token", server.base));
    assert!(credentials.expires_at_ms.is_some());

    let requests = server.requests();
    let token_body = requests
        .iter()
        .find(|(method, path, _)| method == "POST" && path == "/token")
        .unwrap()
        .2
        .clone();
    assert!(token_body.contains("grant_type=authorization_code"));
    assert!(token_body.contains("code=the-code"));
    assert!(token_body.contains("code_verifier="));
    assert!(token_body.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A1%2Fcallback"));
    assert!(token_body.contains("client_id=registered-client"));
    assert!(token_body.contains("resource=http%3A%2F%2F"));

    let registration_body = requests
        .iter()
        .find(|(method, path, _)| method == "POST" && path == "/register")
        .unwrap()
        .2
        .clone();
    let registration: serde_json::Value = serde_json::from_str(&registration_body).unwrap();
    assert_eq!(registration["client_name"].as_str(), Some("test"));
    assert_eq!(
        registration["redirect_uris"][0].as_str(),
        Some("http://127.0.0.1:1/callback")
    );
}

#[test]
fn a_token_error_is_reported() {
    let server = start_server(|method, path, base| match (method, path) {
        ("POST", "/mcp") => (
            401,
            "{}".to_string(),
            vec![(
                "WWW-Authenticate".to_string(),
                format!("Bearer resource_metadata=\"{base}/.well-known/oauth-protected-resource\""),
            )],
        ),
        ("GET", "/.well-known/oauth-protected-resource") => (
            200,
            format!(r#"{{"resource":"{base}/mcp","authorization_servers":["{base}"]}}"#),
            vec![],
        ),
        ("GET", "/.well-known/oauth-authorization-server") => (
            200,
            format!(
                r#"{{"authorization_endpoint":"{base}/authorize","token_endpoint":"{base}/token","registration_endpoint":"{base}/register"}}"#
            ),
            vec![],
        ),
        ("POST", "/register") => (
            200,
            r#"{"client_id":"registered-client"}"#.to_string(),
            vec![],
        ),
        ("POST", "/token") => (400, r#"{"error":"invalid_grant"}"#.to_string(), vec![]),
        _ => (404, "{}".to_string(), vec![]),
    });

    let mcp_server = http_server("mcp", &format!("{}/mcp", server.base));
    let authorization = begin(&mcp_server, "http://127.0.0.1:1/callback", &client_info()).unwrap();
    let error = finish(authorization.pending, "the-code").unwrap_err();
    assert!(error.to_string().contains("400"));
}

#[test]
fn refresh_replaces_the_access_token_and_keeps_a_missing_refresh_token() {
    let server = start_server(|method, path, _| match (method, path) {
        ("POST", "/token") => (
            200,
            r#"{"access_token":"new-access","expires_in":60}"#.to_string(),
            vec![],
        ),
        _ => (404, "{}".to_string(), vec![]),
    });

    let credentials = McpCredentials {
        client_id: "client".into(),
        client_secret: None,
        token_endpoint: format!("{}/token", server.base),
        resource: None,
        access_token: "old".into(),
        refresh_token: Some("r1".into()),
        expires_at_ms: Some(0),
    };

    let refreshed = refresh(&credentials).unwrap();
    let body = server
        .requests()
        .into_iter()
        .find(|(method, path, _)| method == "POST" && path == "/token")
        .unwrap()
        .2;
    assert!(body.contains("grant_type=refresh_token"));
    assert!(body.contains("refresh_token=r1"));
    assert!(body.contains("client_id=client"));
    assert_eq!(refreshed.access_token, "new-access");
    assert_eq!(refreshed.refresh_token.as_deref(), Some("r1"));
    assert!(refreshed.expires_at_ms.is_some());
}

#[test]
fn discovery_falls_back_without_a_challenge() {
    let server = start_server(|method, path, base| match (method, path) {
        ("POST", "/mcp") => (401, "{}".to_string(), vec![]),
        ("GET", "/.well-known/oauth-protected-resource") => (
            200,
            format!(r#"{{"resource":"{base}/mcp","authorization_servers":["{base}"]}}"#),
            vec![],
        ),
        ("GET", "/.well-known/oauth-authorization-server") => (
            200,
            format!(
                r#"{{"authorization_endpoint":"{base}/authorize","token_endpoint":"{base}/token","registration_endpoint":"{base}/register"}}"#
            ),
            vec![],
        ),
        ("POST", "/register") => (
            200,
            r#"{"client_id":"fallback-client"}"#.to_string(),
            vec![],
        ),
        _ => (404, "{}".to_string(), vec![]),
    });

    let mcp_server = http_server("mcp", &format!("{}/mcp", server.base));
    let authorization = begin(&mcp_server, "http://127.0.0.1:1/callback", &client_info()).unwrap();
    assert!(
        authorization
            .url
            .starts_with(&format!("{}/authorize", server.base))
    );
    assert!(authorization.url.contains("client_id=fallback-client"));

    let requests = server.requests();
    assert!(requests.iter().any(
        |(method, path, _)| method == "GET" && path == "/.well-known/oauth-protected-resource"
    ));
}
