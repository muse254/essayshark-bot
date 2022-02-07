use serde::Deserialize;

#[derive(Debug)]
pub enum Discard {
    All,
    Default, // Rewriting_Editing
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
    pub new_items: Option<Vec<NewItem>>,

    #[serde(rename = "qty_discarded")]
    pub qty_discarded: u8,

    #[serde(rename = "qty_filtered")]
    pub qty_filtered: u8,

    #[serde(rename = "qty_total")]
    pub qty_total: u8,
}

#[derive(Deserialize, Default)]
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
    pub discipline2_sub_ar: Option<Discipline2SubAr>,

    #[serde(rename = "paper_type_ar")]
    pub paper_type_ar: PaperTypeAr,

    #[serde(rename = "paper_lang_ar")]
    pub paper_lang_ar: PaperLangAr,

    #[serde(rename = "deadline_dt_fmt")]
    pub deadline_dt_fmt: String,

    #[serde(rename = "min_price_total")]
    pub min_price_total: f32,
}

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
pub struct PaperLangAr {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "pos")]
    pub pos: String,

    #[serde(rename = "title")]
    pub title: String,
}

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
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
