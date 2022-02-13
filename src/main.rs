use bot::types;
use futures::executor::block_on;
use http;
use log::{error, info, trace, warn, LevelFilter};
use reqwest;
use select::document::Document;
use select::predicate::Name;
use simple_logger::SimpleLogger;
use std::future::Future;
use std::pin::Pin;
use tokio;
use url::Url;

const BID_TRIES: u8 = 200;
static COOKIE_FILE: &str = "./src/cookie.txt";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // This allows to filter extra logs such that only error logs are printed.
    SimpleLogger::new()
        .with_module_level("hyper", LevelFilter::Error)
        .with_module_level("mio", LevelFilter::Error)
        .with_module_level("tracing", LevelFilter::Error)
        .with_module_level("want", LevelFilter::Error)
        .with_module_level("reqwest", LevelFilter::Error)
        .with_module_level("html5ever", LevelFilter::Error)
        .init()
        .unwrap();

    // Indicates the start of the program.
    trace!("Program started");

    // Construct client with the propagated auth cookie for all requests.
    let client = reqwest::ClientBuilder::new()
        .default_headers(retrieve_cookie_from_file())
        .build()
        .unwrap();

    match get_orders(client.clone(), 1).await.new_items {
        Some(available_orders) => {
            // discard all available orders
            discard_orders(
                client.clone(),
                available_orders
                    .iter()
                    .map(|order| order.id.to_string())
                    .collect::<Vec<String>>(),
            )
            .await;
        }
        None => {}
    }

    // wait for new orders and bid iteratively
    find_orders_and_bid(&client).await;

    Ok(())
}

fn retrieve_cookie_from_file() -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::COOKIE,
        http::header::HeaderValue::from_str(std::fs::read_to_string(COOKIE_FILE).unwrap().as_str())
            .unwrap(),
    );
    headers
}

fn get_orders(
    client: reqwest::Client,
    tries: u8,
) -> Pin<Box<dyn Future<Output = types::AvailableOrders>>> {
    const URL : &'static str =  "https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_=";

    Box::pin(async move {
        match client.get(URL).send().await {
            Ok(resp) =>
            // fetch and deserialize content
            {
                let orders =
                    serde_json::from_str::<types::AvailableOrders>(&resp.text().await.unwrap())
                        .unwrap();
                info!("#{} order(s) found", {
                    if let Some(item) = &orders.new_items {
                        item.len()
                    } else {
                        0
                    }
                },);

                orders
            }
            Err(err) => {
                warn!("an error occurred retrieving orders: {:?}", err);

                // retry 5 times then exit if failed
                if tries == 5 {
                    error!("{}", err);
                    panic!()
                } else {
                    get_orders(client, tries + 1).await
                }
            }
        }
    })
}

async fn discard_orders(client: reqwest::Client, orders: Vec<String>) {
    // early return
    if orders.is_empty() {
        return;
    }

    match client
        .post("https://essayshark.com/writer/orders/aj_source.html")
        .form(&[
            ("act", "discard_all"),
            ("nobreath", "1"),
            ("ids", &orders.join(",")),
        ])
        .send()
        .await
    {
        Ok(_) => {
            info!("{:?} discarded", orders)
        }
        Err(err) => {
            warn!("an error occurred discarding orders: {}", err);
        }
    }
}

async fn find_orders_and_bid(client: &reqwest::Client) {
    loop {
        info!("finding new orders to bid to");

        // find new orders and bid
        if let Some(available_orders) = get_orders(client.clone(), 1).await.new_items {
            if available_orders.is_empty() {
                continue;
            }

            // bid first
            let mut bids: u8 = 0;
            let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();

            {
                let ids_str = available_orders
                    .iter()
                    .filter(|&item| item.service_type_ar.slug == "editing_rewriting")
                    .map(|item| item.id.clone())
                    .collect::<Vec<String>>();
                let client_clone = client.clone();
                threads.push(std::thread::spawn(move || {
                    // discard editing_rewrite if any needs discarding
                    block_on(discard_orders(client_clone, ids_str))
                }));
            }

            // silence orders with the tag edit_rewrite tag or that have been cached
            for item in available_orders
                .into_iter()
                .filter(|item| item.service_type_ar.slug != "editing_rewriting")
            {
                let client_clone = client.clone();
                // let item_clone = item.copy();
                let item_id = item.id.clone();
                let item_min_price_total = item.min_price_total.clone();
                let item_pages_qty = item.pages_qty.clone();

                // the number of available orders per second won't fly say above 10;
                // so spawning for every order will be relatively safe
                threads.push(std::thread::spawn(move || {
                    block_on({
                        dispatch_order(client_clone, item_id, item_min_price_total, item_pages_qty)
                    })
                }));

                bids += 1;
            }

            if bids > 0 {
                info!("trying to bid {} order(s)", bids);
            }

            // wait for all threads to finish in a seperate thread
            // but continue with program execution; for new orders that might be found
            std::thread::spawn(move || {
                for thread in threads {
                    thread.join().unwrap();
                }
            });
        }
    }
}

async fn dispatch_order<'a>(
    client: reqwest::Client,
    item_id: String,
    min_price_total: f32,
    pages_qty: String,
) {
    if let Some(err) = client
        .post(format!(
            "https://essayshark.com/writer/orders/{}.html",
            item_id
        ))
        .send()
        .await
        .err()
    {
        error!("{}", err);
        panic!()
    }

    info!("order id {} dispatched", item_id);

    queue_bid(
        client.clone(),
        item_id.to_owned(),
        min_price_total,
        pages_qty.to_owned(),
        1,
        false,
    )
    .await
    .await;
}

async fn queue_bid<'a>(
    client: reqwest::Client,
    item_id: String,
    min_price_total: f32,
    pages_qty: String,
    tries: u8,
    download_dispatch: bool,
) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        match client
            .get(format!(
                "https://essayshark.com/writer/orders/ping.html?order={}&_=",
                item_id.clone()
            ))
            .send()
            .await
        {
            Ok(resp) => {
                // deserialize response
                let order_ping =
                    serde_json::from_str::<types::OrderPing>(&resp.text().await.unwrap()).unwrap();

                info!("order #{} pinged", &item_id);

                if order_ping.read_time_remain == 0 {
                    let id = item_id.clone();
                    info!("order #{} bidding", id);
                    submit_bid(&client, item_id.clone(), min_price_total, pages_qty.clone()).await;
                    return;
                } else if order_ping.read_time_remain <= 17 {
                    if order_ping.pr > 0 {
                        submit_bid(&client, item_id.clone(), min_price_total, pages_qty.clone())
                            .await;
                        return;
                    }
                }

                // dispatch the queue asynchronously
                let queue = queue_bid(
                    client.clone(),
                    item_id.clone(),
                    min_price_total,
                    pages_qty,
                    tries + 1,
                    true,
                );

                // dispatch the download asynchronously
                let routine = std::thread::spawn(move || {
                    if !download_dispatch && order_ping.files_download_remain > 0 {
                        block_on(download_file(client, item_id))
                    }
                });

                // make sure the queue completes
                queue.await.await;

                // make sure the download thread completes
                // uninterested in result; the download data is discarded
                let _ = routine.join();
            }
            Err(err) => {
                if tries.eq(&BID_TRIES) {
                    error!("{}", err);
                    panic!()
                }
                queue_bid(
                    client.clone(),
                    item_id,
                    min_price_total,
                    pages_qty,
                    tries + 1,
                    download_dispatch,
                )
                .await
                .await
            }
        }
    })
}

async fn download_file<'a>(client: reqwest::Client, order_id: String) {
    match client
        .get(format!(
            "https://essayshark.com/writer/orders/{}.html",
            order_id
        ))
        .send()
        .await
    {
        Ok(resp) => {
            // retrieve document body & parse as html
            let body = resp.text().await.unwrap();
            let body_owned = body.as_str();
            {
                let doc = Document::from(body_owned);
                let document = doc
                    .find(Name("a"))
                    .filter(|&node| {
                        // a[target="download_ifm"][href]
                        if let Some(target) = node.attr("target") {
                            target == "download_ifm"
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<select::node::Node>>();

                async fn download_resource_background(client_owned: reqwest::Client, path: String) {
                    // parse url from base
                    let base = Url::parse("https://essayshark.com")
                        .expect("hardcoded URL is known to be valid");
                    let joined = base.join(&path);

                    match joined {
                        Ok(url) => {
                            if let Some(err) = client_owned.get(url.to_string()).send().await.err()
                            {
                                warn!("{}", err);
                            }
                            info!("download for url {} completed!", url);
                        }
                        Err(err) => {
                            warn!("{}", err);
                        }
                    }
                }

                // only need to 'download' a single file
                if let Some(node) = document.first() {
                    let client_clone = client.clone();
                    if let Some(url) = node.attr("href") {
                        download_resource_background(client_clone, url.to_string()).await;
                    }
                }
            }
        }
        Err(err) => {
            error!("{}", err);
        }
    }
}

async fn submit_bid<'a>(
    client: &reqwest::Client,
    order_id: String,
    min_price_total: f32,
    pages_qty: String,
) {
    match client
        .post(format!(
            "https://essayshark.com/writer/orders/{}.html",
            &order_id
        ))
        .form(&[
            ("bid_add_ua", "mmmmmm"),
            ("bid_add", "1"),
            (
                "bid",
                &format!("{}", (min_price_total / pages_qty.parse::<f32>().unwrap())),
            ),
        ])
        .send()
        .await
    {
        Ok(_) => {
            // info!("{:?}", resp);
            info!("bid submitted for item id: {} ", order_id);
        }
        Err(err) => {
            error!("{}", err);
            panic!()
        } //info!("bid successfully submitted for item id: {} ", order_id);
    }
}
