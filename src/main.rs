use cloudflare_bypasser;
use http;
use http::HeaderMap;
use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // do the login & retrieve auth cookie
    let auth_cookie = do_login().await;

    // construct client with the propagated auth cookie for all requests.
    let client = reqwest::ClientBuilder::new()
        .default_headers(auth_cookie)
        .build()
        .unwrap();

    // discard all available orders
    discard_orders(&client, types::Discard::All).await;

    // wait for new orders and bid
    find_orders_and_bid(&client).await;

    Ok(())
}

async fn do_login() -> HeaderMap {
    const LOGIN_URL: &'static str = "https://essayshark.com/auth/aj_login2.html?callback=";

    // create a client
    let client = reqwest::ClientBuilder::new().build().unwrap();

    // post with formdata
    match client
        .post(LOGIN_URL)
        .form(&[
            ("l", "cmutungi17@yahoo.com"),
            ("p", "Log@nj@b@li2020"),
            ("stay_signed_in", "1"),
            ("marketing", "0"),
            ("policy", "0"),
            ("role", ""),
        ])
        .send()
        .await
    {
        Ok(resp) => {
            let mut h_map = HeaderMap::new();
            h_map.insert(
                http::header::COOKIE,
                resp.headers()
                    .get("set-cookie")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
            return h_map;
        }

        Err(err) => {
            // There's no need to log the error, just panic!
            panic!("{}", err);
        }
    }
}

async fn get_orders(client: &reqwest::Client) -> types::AvailableOrders {
    const URL : &'static str =  "https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_=";

    match client.get(URL).send().await {
        Ok(resp) => {
            // fetch and deserialize content
            serde_json::from_str::<types::AvailableOrders>(&resp.text().await.unwrap()).unwrap()
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}

async fn discard_orders(client: &reqwest::Client, opts: types::Discard) {
    // get all available order items
    let available_orders = get_orders(&client).await.new_items;

    // get ids and collect to array
    let ids_str = available_orders
        .iter()
        .filter(|&item| match &opts {
            types::Discard::All => true,
            types::Discard::Default => item.service_type_ar.slug == "editing_rewriting",
        })
        .map(|item| item.id.as_str())
        .collect::<Vec<&str>>();

    // construct json string
    let ids_json = json!(ids_str).to_string();

    if let Some(err) = client
        .post("https://essayshark.com/writer/orders/aj_source.html")
        .form(&[
            ("act", "discard_all"),
            ("nobreath", "1"),
            ("ids", &ids_json),
        ])
        .send()
        .await
        .err()
    {
        panic!("{}", err)
    }
}

async fn find_orders_and_bid(client: &reqwest::Client) {
    // find new orders and bid

    // discard orders with the tag edit_rewrite tag

    // sleep 0.1 s, repeat
}

async fn dispatch_order<'a>(
    client: &reqwest::Client,
    item_id: &'a str,
    min_price_total: &f32,
    pages_qty: &'a str,
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
        panic!("{}", err)
    }

    queue_bid(client, item_id, min_price_total, pages_qty).await;
}

async fn queue_bid<'a>(
    client: &reqwest::Client,
    item_id: &'a str,
    min_price_total: &f32,
    pages_qty: &'a str,
) {
    match client
        .get(format!(
            "https://essayshark.com/writer/orders/ping.html?order={}&_=",
            item_id
        ))
        .send()
        .await
    {
        Ok(resp) => {
            // deserialize response
            let order_ping =
                serde_json::from_str::<types::OrderPing>(&resp.text().await.unwrap()).unwrap();

            if order_ping.read_time_remain == 0 {
                submit_bid(&client, item_id, min_price_total, pages_qty).await;
            } else if order_ping.read_time_remain == 10 {
                if order_ping.pr > 0 {
                    submit_bid(&client, item_id, min_price_total, pages_qty).await;
                    if let Some(err) = client
                        .get(format!(
                            "https://essayshark.com/writer/orders/ping.html?order={}&_=",
                            item_id
                        ))
                        .send()
                        .await
                        .err()
                    {
                        panic!("{}", err);
                    }
                }
            } else {
            }
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}

async fn dispatch_orders(client: &reqwest::Client, new_items: Vec<types::NewItem>) {
    if new_items.is_empty() {
        return find_orders().await;
    }

    for i in new_items {
        if i.order_read != "N" || {
            if let Some(val) = &i.bid_outdated {
                val == "Y"
            } else {
                false
            }
        } {
            return find_orders().await;
        } else {
            dispatch_order(client, &i.id, &i.min_price_total, &i.pages_qty).await;
        }
    }
}

async fn submit_bid<'a>(
    client: &reqwest::Client,
    order_id: &'a str,
    min_price_total: &f32,
    pages_qty: &'a str,
) {
    if let Some(err) = client
        .post(format!(
            "https://essayshark.com/writer/orders/{}.html",
            order_id
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
        .err()
    {
        panic!("{}", err)
    }
}

mod types {
    use serde::Deserialize;

    pub enum Discard {
        All,
        Default, // Rewriting/Editing
    }

    #[derive(Deserialize)]
    pub struct OrderPing {
        pub read_time_remain: u32,
        pub files_download_remain: u8,
        pub pr: u32,
    }

    #[derive(Deserialize)]
    pub struct AvailableOrders {
        #[serde(rename = "filtered")]
        pub filtered: u8,

        #[serde(rename = "new_items")]
        pub new_items: Vec<NewItem>,

        #[serde(rename = "qty_discarded")]
        pub qty_discarded: u8,

        #[serde(rename = "qty_filtered")]
        pub qty_filtered: u8,

        #[serde(rename = "qty_total")]
        pub qty_total: u8,
    }

    #[derive(Deserialize)]
    pub struct NewItem {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "trusted")]
        pub trusted: String,

        #[serde(rename = "writer_req")]
        pub writer_req: String,

        #[serde(rename = "cur_writer_req")]
        pub cur_writer_req: String,

        #[serde(rename = "title")]
        pub title: String,

        #[serde(rename = "pages_qty")]
        pub pages_qty: String,

        #[serde(rename = "service_type")]
        pub service_type: String,

        #[serde(rename = "budget_req")]
        pub budget_req: String,

        #[serde(rename = "amount_writer")]
        pub amount_writer: String,

        #[serde(rename = "price_type")]
        pub price_type: String,

        #[serde(rename = "paper_type")]
        pub paper_type: String,

        #[serde(rename = "paper_type_txt")]
        pub paper_type_txt: String,

        #[serde(rename = "paper_lang")]
        pub paper_lang: String,

        #[serde(rename = "academic_level")]
        pub academic_level: String,

        #[serde(rename = "discipline2")]
        pub discipline2: String,

        #[serde(rename = "discipline2_sub")]
        pub discipline2_sub: String,

        #[serde(rename = "discipline2_txt")]
        pub discipline2_txt: String,

        #[serde(rename = "deadline_dt_ts")]
        pub deadline_dt_ts: String,

        #[serde(rename = "late")]
        pub late: String,

        #[serde(rename = "featured")]
        pub featured: String,

        #[serde(rename = "customer_debut")]
        pub customer_debut: String,

        #[serde(rename = "discarded4writer")]
        pub discarded4_writer: u8,

        #[serde(rename = "customer_rating")]
        pub customer_rating: String,

        #[serde(rename = "customer_orders")]
        pub customer_orders: String,

        #[serde(rename = "translation_lang_from")]
        pub translation_lang_from: String,

        #[serde(rename = "translation_lang_from_txt")]
        pub translation_lang_from_txt: String,

        #[serde(rename = "translation_lang_to")]
        pub translation_lang_to: String,

        #[serde(rename = "translation_lang_to_txt")]
        pub translation_lang_to_txt: String,

        #[serde(rename = "translation_chars_qty")]
        pub translation_chars_qty: String,

        #[serde(rename = "order_read")]
        pub order_read: String,

        #[serde(rename = "bid_outdated")]
        pub bid_outdated: Option<String>,

        #[serde(rename = "online_status")]
        pub online_status: String,

        #[serde(rename = "bids_qty")]
        pub bids_qty: u32,

        #[serde(rename = "status_ar")]
        pub status_ar: Vec<Option<String>>,

        #[serde(rename = "status_prev_ar")]
        pub status_prev_ar: Vec<Option<String>>,

        #[serde(rename = "service_type_ar")]
        pub service_type_ar: ServiceTypeAr,

        #[serde(rename = "discipline2_ar")]
        pub discipline2_ar: Discipline2Ar,

        #[serde(rename = "discipline2_sub_ar")]
        pub discipline2_sub_ar: Discipline2SubAr,

        #[serde(rename = "paper_type_ar")]
        pub paper_type_ar: PaperTypeAr,

        #[serde(rename = "paper_lang_ar")]
        pub paper_lang_ar: PaperLangAr,

        #[serde(rename = "deadline_dt_fmt")]
        pub deadline_dt_fmt: String,

        #[serde(rename = "min_price_total")]
        pub min_price_total: f32,
    }

    #[derive(Deserialize)]
    pub struct Discipline2Ar {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "pos")]
        pub pos: Option<String>,

        #[serde(rename = "site_lang")]
        pub site_lang: Option<String>,

        #[serde(rename = "profile")]
        pub profile: Option<String>,

        #[serde(rename = "slug")]
        pub slug: Option<String>,

        #[serde(rename = "url_part")]
        pub url_part: Option<String>,

        #[serde(rename = "title")]
        pub title: String,
    }

    #[derive(Deserialize)]
    pub struct Discipline2SubAr {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "discipline")]
        pub discipline: Option<String>,

        #[serde(rename = "pos")]
        pub pos: Option<String>,

        #[serde(rename = "pos_in_popular")]
        pub pos_in_popular: Option<String>,

        #[serde(rename = "title")]
        pub title: String,
    }

    #[derive(Deserialize)]
    pub struct PaperLangAr {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "pos")]
        pub pos: String,

        #[serde(rename = "title")]
        pub title: String,
    }

    #[derive(Deserialize)]
    pub struct PaperTypeAr {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "title")]
        pub title: String,

        #[serde(rename = "service_type")]
        pub service_type: Option<String>,

        #[serde(rename = "pos")]
        pub pos: Option<String>,

        #[serde(rename = "site_lang")]
        pub site_lang: Option<String>,

        #[serde(rename = "guide_file")]
        pub guide_file: Option<String>,

        #[serde(rename = "enabled")]
        pub enabled: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct ServiceTypeAr {
        #[serde(rename = "id")]
        pub id: String,

        #[serde(rename = "pos")]
        pub pos: String,

        #[serde(rename = "site_lang")]
        pub site_lang: String,

        #[serde(rename = "slug")]
        pub slug: String,

        #[serde(rename = "category_slug")]
        pub category_slug: String,

        #[serde(rename = "layout_type")]
        pub layout_type: String,

        #[serde(rename = "attach_file_required")]
        pub attach_file_required: String,

        #[serde(rename = "title")]
        pub title: String,

        #[serde(rename = "comment")]
        pub comment: String,
    }
}
