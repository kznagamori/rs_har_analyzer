//! HARファイルの構造を定義するデータ型

use serde::{Deserialize, Serialize};

/// HARファイルのルート構造
#[derive(Debug, Deserialize, Serialize)]
pub struct HarFile {
    pub log: Log,
}

/// HARログのメイン構造
#[derive(Debug, Deserialize, Serialize)]
pub struct Log {
    pub version: String,
    pub creator: Creator,
    pub entries: Vec<Entry>,
}

/// HARファイルを作成したツールの情報
#[derive(Debug, Deserialize, Serialize)]
pub struct Creator {
    pub name: String,
    pub version: String,
}

/// HTTPリクエスト/レスポンスのエントリ
#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    #[serde(rename = "startedDateTime")]
    pub started_date_time: String,
    pub time: f64,
    pub request: Request,
    pub response: Response,
    #[serde(default)]
    pub cache: Cache,
    pub timings: Timings,
    #[serde(rename = "serverIPAddress")]
    pub server_ip_address: Option<String>,
    pub connection: Option<String>,
}

/// HTTPリクエストの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(default)]
    pub headers: Vec<NameValue>,
    #[serde(rename = "queryString", default)]
    pub query_string: Vec<NameValue>,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(rename = "headersSize")]
    pub headers_size: i64,
    #[serde(rename = "bodySize")]
    pub body_size: i64,
    #[serde(rename = "postData")]
    pub post_data: Option<PostData>,
}

/// HTTPレスポンスの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub status: i32,
    #[serde(rename = "statusText")]
    pub status_text: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(default)]
    pub headers: Vec<NameValue>,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    pub content: Content,
    #[serde(rename = "redirectURL")]
    pub redirect_url: String,
    #[serde(rename = "headersSize")]
    pub headers_size: i64,
    #[serde(rename = "bodySize")]
    pub body_size: i64,
}

/// 名前と値のペア
#[derive(Debug, Deserialize, Serialize)]
pub struct NameValue {
    pub name: String,
    pub value: String,
}

/// クッキーの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expires: Option<String>,
    #[serde(rename = "httpOnly")]
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
}

/// POSTデータの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct PostData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(default)]
    pub params: Vec<Param>,
    pub text: Option<String>,
}

/// パラメータの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct Param {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
}

/// レスポンスコンテンツの詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    pub size: i64,
    pub compression: Option<i64>,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: Option<String>,
    pub encoding: Option<String>,
}

/// キャッシュ情報
#[derive(Debug, Deserialize, Serialize)]
pub struct Cache {
    #[serde(rename = "beforeRequest")]
    pub before_request: Option<CacheState>,
    #[serde(rename = "afterRequest")]
    pub after_request: Option<CacheState>,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            before_request: None,
            after_request: None,
        }
    }
}

/// キャッシュ状態
#[derive(Debug, Deserialize, Serialize)]
pub struct CacheState {
    #[serde(rename = "lastAccess")]
    pub last_access: String,
    #[serde(rename = "eTag")]
    pub etag: String,
    #[serde(rename = "hitCount")]
    pub hit_count: i32,
}

/// タイミング情報
#[derive(Debug, Deserialize, Serialize)]
pub struct Timings {
    pub blocked: Option<f64>,
    pub dns: Option<f64>,
    pub connect: Option<f64>,
    pub send: f64,
    pub wait: f64,
    pub receive: f64,
    pub ssl: Option<f64>,
}

impl Default for Timings {
    fn default() -> Self {
        Timings {
            blocked: None,
            dns: None,
            connect: None,
            send: 0.0,
            wait: 0.0,
            receive: 0.0,
            ssl: None,
        }
    }
}

/// 解析結果を格納するための構造体
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub timestamp: String,
    pub source_ip: String,
    pub destination_ip: String,
    pub method: String,
    pub status_code: i32,
    pub request_url: String,
    pub request_payload: String,
    pub response_payload: String,
}