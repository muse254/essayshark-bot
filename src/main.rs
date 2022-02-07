use bot::types;
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
    SimpleLogger::new()
        .with_module_level("hyper", LevelFilter::Error)
        .with_module_level("mio", LevelFilter::Error)
        .with_module_level("tracing", LevelFilter::Error)
        .with_module_level("want", LevelFilter::Error)
        .with_module_level("reqwest", LevelFilter::Error)
        .with_module_level("html5ever", LevelFilter::Error)
        .init()
        .unwrap();

    trace!("Program started");

    // construct client with the propagated auth cookie for all requests.
    let client = reqwest::ClientBuilder::new()
        .default_headers(retrieve_cookie_from_file())
        .build()
        .unwrap();

    match get_orders(client.clone(), 1).await.new_items {
        Some(available_orders) => {
            // discard all available orders
            discard_orders(&client, &available_orders, types::Discard::All).await;
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

async fn discard_orders(
    client: &reqwest::Client,
    available_orders: &Vec<types::NewItem>,
    opts: types::Discard,
) {
    // early return
    if available_orders.is_empty() {
        return;
    }

    // get ids and collect to array
    let ids_str = available_orders
        .iter()
        .filter(|&item| match &opts {
            types::Discard::All => true,
            types::Discard::Default => item.service_type_ar.slug == "editing_rewriting",
        })
        .map(|item| item.id.as_str())
        .collect::<Vec<&str>>();

    // early return
    if ids_str.is_empty() {
        return;
    }

    match client
        .post("https://essayshark.com/writer/orders/aj_source.html")
        .form(&[
            ("act", "discard_all"),
            ("nobreath", "1"),
            ("ids", &ids_str.join(",")),
        ])
        .send()
        .await
    {
        Ok(_) => {
            info!("{:?} discarded", ids_str)
        }
        Err(err) => {
            warn!("an error occurred discarding orders: {}", err);
        }
    }
    info!("available orders with opts {:?} discarded", opts)
}

async fn find_orders_and_bid(client: &reqwest::Client) {
    let mut cache = Vec::new();

    loop {
        info!("finding new orders to bid to");
        // find new orders and bid
        if let Some(available_orders) = get_orders(client.clone(), 1).await.new_items {
            // silence orders with the tag edit_rewrite tag or that have been cached
            let ids_str = available_orders
                .iter()
                .filter(|&item| {
                    item.service_type_ar.slug != "editing_rewriting" && !cache.contains(&item.id)
                })
                .map(|item| item.id.as_str())
                .collect::<Vec<&str>>();

            // bid if there are available orders.
            if !ids_str.is_empty() {
                // bid first
                let mut bids: u8 = 0;
                for item in available_orders
                    .iter()
                    .skip_while(|&x| !ids_str.contains(&x.id.as_str()))
                {
                    let client_clone = client.clone();
                    dispatch_order(
                        client_clone,
                        item.id.clone(),
                        item.min_price_total,
                        item.pages_qty.clone(),
                    )
                    .await;
                    cache.push(item.id.clone());
                    bids += 1;
                }

                if bids > 0 {
                    info!("trying to bid {} order(s)", bids);
                }
            } else {
                // no availble orders
                info!("no orders to process");
            }
            // discard editing_rewrite if any needs discarding
            discard_orders(&client, &available_orders, types::Discard::Default).await;
        } else {
            warn!("orders found dropped");
        }

        // sleep 1 ms then continue loop
        // thread::sleep(time::Duration::from_millis(1));
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

                if order_ping.files_download_remain > 0 {
                    download_file(&client, &item_id).await;
                }

                //recurse
                queue_bid(
                    client.clone(),
                    item_id.clone(),
                    min_price_total,
                    pages_qty,
                    tries + 1,
                )
                .await
                .await
            }
            Err(err) => {
                if tries.eq(&BID_TRIES) {
                    error!("{}", err);
                    panic!()
                } else {
                    queue_bid(client.clone(), item_id, min_price_total, pages_qty, tries)
                        .await
                        .await
                }
            }
        }
    })
}

async fn download_file<'a>(client: &reqwest::Client, order_id: &'a str) {
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
                                //    panic!()
                            }
                            info!("download for url {} completed!", url);
                        }
                        Err(err) => {
                            warn!("{}", err);
                            //  panic!()
                        }
                    }
                }

                'this: for node in document.into_iter() {
                    let client_clone = client.clone();
                    if let Some(url) = node.attr("href") {
                        download_resource_background(client_clone, url.to_string()).await;
                        break 'this;
                    }
                }
            }
        }
        Err(err) => {
            error!("{}", err);
            // panic!()
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
