#![deny(warnings)]

use std::convert::Infallible;

use std::collections::HashMap;
use url::Url;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

extern crate pretty_env_logger;

async fn proxy(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    // A reusable response for returning 500 errors
    let response500 = Response::builder()
        .status(500)
        .body(Body::from("Proxy request failed"))
        .unwrap();

    // The incoming request path, including the query string
    let request_path = request.uri().to_string();

    // Create a complete URL with the request path, using an arbitrary domain
    let fake_url = format!("http://example.com{}", request_path);
    let fake_url_str: &str = &*fake_url;

    // If the path is empty, do nothing
    if request_path == "/" {
        return Ok(Response::new(Body::from("OK")));
    }

    // Parse the full URL
    let request_url = match Url::parse(fake_url_str) {
        Ok(u) => u,
        Err(_) => {
            println!("Couldn't parse URL: {}", fake_url_str);
            return Ok(response500);
        },
    };

    // Construct a map of the query string key-values
    let params = request_url.query_pairs();
    let hash_query: HashMap<_, _> = params.into_owned().collect();

    // Extract the `url` query parameter, which should be the complete feed
    // URL needing to be proxied
    let feed_url = match hash_query.get("url") {
        Some(u) => u,
        None => {
            println!("Missing URL query parameter: {}", fake_url_str);
            return Ok(response500);
        }
    };

    println!("URL: {}", feed_url);

    // GET the feed with a reqwest client
    let client = reqwest::Client::new();
    let res = match client
        .get(feed_url)
        .header("User-Agent", "play.prx.org feed proxy")
        .header("Accept", "application/rss+xml,application/rdf+xml;q=0.8,application/atom+xml;q=0.6,application/xml;q=0.4,text/xml;q=0.4")
        .send()
        .await {
            Ok(r) => r,
            Err(err) =>  {
                println!("Bad origin request: {}", fake_url_str);
                println!("{}", err);
                return Ok(response500);
            },
        };

    // Get the response body from the feed request
    let content = match res
        .text()
        .await {
            Ok(r) => r,
            Err(err) =>  {
                println!("Invalid origin content: {}", fake_url_str);
                println!("{}", err);
                return Ok(response500);
            },
        };

    // Return the feed response as the proxied response
    return Ok(Response::builder()
        .status(200)
        .header("Cache-Control", "public, max-age=90")
        .header("Server", "play-proxy/hyper")
        .body(Body::from(content))
        .unwrap());
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

    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
