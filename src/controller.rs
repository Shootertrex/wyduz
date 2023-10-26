use std::str::FromStr;

use ureq::{Agent, AgentBuilder};

#[derive(Debug)]
pub enum HttpMethod {
    Delete,
    Get,
    Post,
    Put,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(input: &str) -> Result<HttpMethod, Self::Err> {
        match input {
            "DELETE" => Ok(HttpMethod::Delete),
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

pub fn build_agent() -> Agent {
    AgentBuilder::new()
        .try_proxy_from_env(true)
        .build()
}

pub fn make_request(agent: &Agent, request: &Request) -> Result<Response, Box<ureq::Error>> {
    match request.method {
        HttpMethod::Get => Ok(make_get_request(agent, request)?),
        HttpMethod::Post => Ok(make_post_request(agent, request)?),
        _ => {
            todo!()
        }
    }
}

fn add_headers(user_request: &Request, mut request: ureq::Request) -> ureq::Request {
    for header in user_request.headers.iter() {
        if header.0.is_empty() || header.1.is_empty() {
            continue;
        }
        request = request.set(&header.0, &header.1);
    }

    request
}

fn build_response_headers(response: &ureq::Response) -> Vec<(String, String)> {
    response
        .headers_names()
        .iter()
        .map(|x| (x.to_owned(), response.header(x).unwrap().to_owned()))
        .collect()
}

fn make_get_request(agent: &Agent, request: &Request) -> Result<Response, Box<ureq::Error>> {
    let mut u_request: ureq::Request = agent.get(&request.url);
    u_request = add_headers(request, u_request);

    let response = u_request.call()?;
    let headers = build_response_headers(&response);

    Ok(Response {
        status_code: response.status(),
        body: response.into_string().expect("failed to get response body"),
        headers,
    })
}

fn make_post_request(agent: &Agent, request: &Request) -> Result<Response, Box<ureq::Error>> {
    let body = &request.body;
    let mut u_request: ureq::Request = agent.post(&request.url);
    u_request = add_headers(request, u_request);

    let response = u_request.send_string(&body.clone().unwrap())?;
    let headers = build_response_headers(&response);

    Ok(Response {
        status_code: response.status(),
        body: response.into_string().expect("failed to get response body"),
        headers,
    })
}

#[cfg(test)]
mod tests {
    use crate::controller::{make_request, HttpMethod, Request, Response, build_agent};

    fn assert_headers(expected_response: &Response, actual_response: &Response) {
        for header in expected_response.headers.iter() {
            assert!(actual_response.headers.contains(&header));
        }
    }

    #[test]
    fn ensure_200_get_response() {
        let mut server = mockito::Server::new();
        let url = server.url();
        server
            .mock("GET", "/")
            .match_header("Accept", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"Franklin\"}")
            .create();
        let request = Request {
            method: HttpMethod::Get,
            url: String::from(url),
            headers: vec![("Accept".into(), "application/json".into())],
            body: None,
        };
        let expected_response = Response {
            status_code: 200,
            body: String::from("{\"name\":\"Franklin\"}"),
            headers: vec![("content-type".into(), "application/json".into())],
        };

        let actual_response: Response = make_request(&build_agent(), &request).unwrap();

        assert_eq!(expected_response.body, actual_response.body);
        assert_headers(&expected_response, &actual_response);
    }

    #[test]
    fn ensure_get_with_empty_string_headers() {
        let mut server = mockito::Server::new();
        let url = server.url();
        server
            .mock("GET", "/")
            .with_status(202)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"Franklin\"}")
            .create();
        let request = Request {
            method: HttpMethod::Get,
            url: String::from(url),
            headers: vec![("".into(), "".into())],
            body: None,
        };
        let expected_response = Response {
            status_code: 202,
            body: String::from("{\"name\":\"Franklin\"}"),
            headers: vec![("content-type".into(), "application/json".into())],
        };

        let actual_response: Response = make_request(&build_agent(), &request).unwrap();

        assert_eq!(expected_response.body, actual_response.body);
        assert_headers(&expected_response, &actual_response);
    }

    #[test]
    fn ensure_html_get_response() {
        let mut server = mockito::Server::new();
        let url = server.url();
        server
            .mock("GET", "/")
            .match_header("Accept", "text/html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><div>text</div></html>")
            .create();
        let request = Request {
            method: HttpMethod::Get,
            url: String::from(url),
            headers: vec![("Accept".into(), "text/html".into())],
            body: None,
        };
        let expected_response = Response {
            status_code: 200,
            body: String::from("<html><div>text</div></html>"),
            headers: vec![("content-type".into(), "text/html".into())],
        };

        let actual_response: Response = make_request(&build_agent(), &request).unwrap();

        assert_eq!(expected_response.body, actual_response.body);
        assert_headers(&expected_response, &actual_response);
    }

    #[test]
    fn ensure_404_get_response() {
        let mut server = mockito::Server::new();
        let url = server.url();
        server
            .mock("GET", "/")
            .match_header("Accept", "application/json")
            .with_status(404)
            .create();
        let request = Request {
            method: HttpMethod::Get,
            url: String::from(url),
            headers: vec![("Accept".into(), "application/json".into())],
            body: None,
        };

        let actual_response = make_request(&build_agent(), &request).unwrap_err();

        let response = actual_response.into_response().unwrap();
        assert_eq!(404, response.status());
        assert_eq!("Not Found", response.status_text());
    }

    #[test]
    fn ensure_full_post_response() {
        let mut server = mockito::Server::new();
        let url = server.url();
        server
            .mock("POST", "/")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("")
            .create();
        let request = Request {
            method: HttpMethod::Post,
            url: String::from(url),
            headers: vec![("content-type".into(), "application/json".into())],
            body: Some(String::from("{\"name\":\"Franklin\"}")),
        };
        let expected_response = Response {
            status_code: 200,
            body: String::from(""),
            headers: vec![("content-type".into(), "application/json".into())],
        };

        let actual_response: Response = make_request(&build_agent(), &request).unwrap();

        assert_eq!(expected_response.body, actual_response.body);
        assert_headers(&expected_response, &actual_response);
    }
}
