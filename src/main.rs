#![deny(warnings)]

use std::convert::Infallible;

use std::collections::HashMap;
use url::Url;

use hyper::Client;
use hyper_tls::HttpsConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Method};

extern crate pretty_env_logger;

async fn proxy(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response500 = Response::builder()
        .status(200)
        .body(Body::from("Proxy request failed"))
        .unwrap();

    let request_path = request.uri().to_string();
    let fake_url = format!("http://example.com{}", request_path);
    let fake_url_str: &str = &*fake_url;

    if request_path == "/" {
        return Ok(Response::new(Body::from("OK")));
    }

    let request_url = match Url::parse(fake_url_str) {
        Ok(u) => u,
        Err(_) => {
            println!("Couldn't parse URL: {}", fake_url_str);
            return Ok(response500);
        },
    };

    let params = request_url.query_pairs();
    let hash_query: HashMap<_, _> = params.into_owned().collect();

    let feed_url = match hash_query.get("url") {
        Some(u) => u,
        None => {
            println!("Missing URL query parameter: {}", fake_url_str);
            return Ok(response500);
        }
    };

    let parsed_feed_url = match feed_url.parse::<hyper::Uri>() {
        Ok(u) => u,
        Err(_) => {
            println!("URL not parsed: {}", fake_url_str);
            return Ok(response500);
        },
    };

    let scheme = match parsed_feed_url.scheme_str() {
        Some(s) => s,
        None => {
            println!("Feed URL missing scheme: {}", fake_url_str);
            return Ok(response500);
        },
    };

    let req = Request::builder()
        .method(Method::GET)
        .uri(feed_url)
        .header("User-Agent", "play.prx.org feed proxy")
        .header("Accept", "application/rss+xml,application/rdf+xml;q=0.8,application/atom+xml;q=0.6,application/xml;q=0.4,text/xml;q=0.4")
        .body(Body::from("")).unwrap();

    println!("URL: {}", feed_url);

    if scheme == "http" {
        let client = Client::new();

        let res = match client.request(req).await {
            Ok(r) => r,
            Err(_) =>  {
                println!("Bad origin request: {}", fake_url_str);
                return Ok(response500);
            },
        };

        return Ok(Response::builder()
            .status(200)
            .header("Cache-Control", "public, max-age=90")
            .body(res.into_body())
            .unwrap());
    } else if scheme == "https" {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let res = match client.request(req).await {
            Ok(r) => r,
            Err(_) =>  {
                println!("Bad origin request: {}", fake_url_str);
                return Ok(response500);
            },
        };

        return Ok(Response::builder()
            .status(200)
            .header("Cache-Control", "public, max-age=90")
            .header("Server", "play-proxy/hyper")
            .body(res.into_body())
            .unwrap());
    } else {
        return Ok(response500);
    };
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(proxy)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
