//! HARファイルの解析処理を実装

use crate::har_types::{AnalysisResult, HarFile};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::DateTime;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use url::Url;

/// HARファイルアナライザ
pub struct HarAnalyzer {
    har_data: HarFile,
}

impl HarAnalyzer {
    /// HARファイルを読み込んで新しいアナライザを作成
    /// 
    /// # Arguments
    /// * `file_path` - HARファイルのパス
    /// 
    /// # Returns
    /// * `Result<HarAnalyzer>` - 成功時はアナライザ、失敗時はエラー
    pub fn new(file_path: &str) -> Result<Self> {
        info!("HARファイルを読み込んでいます: {}", file_path);
        
        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow!("ファイルの読み込みに失敗しました: {}", e))?;
        
        debug!("ファイルサイズ: {} bytes", content.len());
        
        let har_data: HarFile = serde_json::from_str(&content)
            .map_err(|e| {
                let error_msg = format!("JSONの解析に失敗しました: {}", e);
                // より詳細なエラー情報を出力
                warn!("エラーが発生した行: {}", e.line());
                warn!("エラーが発生した列: {}", e.column());
                anyhow!(error_msg)
            })?;
        
        info!("HARファイルの読み込みが完了しました。エントリ数: {}", har_data.log.entries.len());
        
        Ok(HarAnalyzer { har_data })
    }

    /// HARファイルを解析して結果を取得
    /// 
    /// # Returns
    /// * `Result<Vec<AnalysisResult>>` - 解析結果のリスト
    pub fn analyze(&self) -> Result<Vec<AnalysisResult>> {
        info!("HARファイルの解析を開始します");
        
        let mut results = Vec::new();
        
        for entry in &self.har_data.log.entries {
            debug!("エントリを処理中: {} {}", entry.request.method, entry.request.url);
            
            // GET/POSTリクエストのみを処理
            if !matches!(entry.request.method.as_str(), "GET" | "POST") {
                debug!("スキップ: {} メソッドは対象外", entry.request.method);
                continue;
            }
            
            // 時刻の解析
            let timestamp = self.parse_timestamp(&entry.started_date_time)?;
            
            // IPアドレスの取得
            let (source_ip, destination_ip) = self.extract_ip_addresses(entry)?;
            
            // リクエストペイロードの取得
            let request_payload = self.extract_request_payload(entry);
            
            // レスポンスペイロードの取得
            let response_payload = self.extract_response_payload(entry);
            
            let result = AnalysisResult {
                timestamp,
                source_ip,
                destination_ip,
                method: entry.request.method.clone(),
                status_code: entry.response.status,
                request_url: self.decode_url(&entry.request.url),
                request_payload,
                response_payload,
            };
            
            results.push(result);
        }
        
        info!("解析が完了しました。結果数: {}", results.len());
        Ok(results)
    }

    /// タイムスタンプを解析してフォーマット
    /// 
    /// # Arguments
    /// * `timestamp_str` - タイムスタンプ文字列
    /// 
    /// # Returns
    /// * `Result<String>` - フォーマットされたタイムスタンプ
    fn parse_timestamp(&self, timestamp_str: &str) -> Result<String> {
        match DateTime::parse_from_rfc3339(timestamp_str) {
            Ok(dt) => Ok(dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
            Err(_) => {
                warn!("タイムスタンプの解析に失敗しました: {}", timestamp_str);
                Ok(timestamp_str.to_string())
            }
        }
    }

    /// IPアドレスを抽出
    /// 
    /// # Arguments
    /// * `entry` - HARエントリ
    /// 
    /// # Returns
    /// * `Result<(String, String)>` - (送信元IP, 送信先IP)
    fn extract_ip_addresses(&self, entry: &crate::har_types::Entry) -> Result<(String, String)> {
        // 送信元IPは通常、HARファイルには含まれないため、プレースホルダーを使用
        let source_ip = "localhost".to_string();
        
        // 送信先IPはserverIPAddressフィールドまたはURLのホスト名から取得
        let destination_ip = if let Some(server_ip) = &entry.server_ip_address {
            server_ip.clone()
        } else {
            // URLからホスト名を抽出
            match Url::parse(&entry.request.url) {
                Ok(url) => {
                    if let Some(host) = url.host_str() {
                        host.to_string()
                    } else {
                        "unknown".to_string()
                    }
                }
                Err(_) => "unknown".to_string(),
            }
        };
        
        Ok((source_ip, destination_ip))
    }

    /// リクエストペイロードを抽出
    /// 
    /// # Arguments
    /// * `entry` - HARエントリ
    /// 
    /// # Returns
    /// * `String` - リクエストペイロード（JSON形式）
    fn extract_request_payload(&self, entry: &crate::har_types::Entry) -> String {
        if let Some(post_data) = &entry.request.post_data {
            if let Some(text) = &post_data.text {
                // JSONかどうかを確認
                if self.is_json_content(&post_data.mime_type) {
                    return self.format_json(text);
                }
                return text.clone();
            }
            
            // パラメータからJSONを構築
            if !post_data.params.is_empty() {
                let mut param_map = HashMap::new();
                for param in &post_data.params {
                    param_map.insert(param.name.clone(), param.value.clone().unwrap_or_default());
                }
                
                match serde_json::to_string_pretty(&param_map) {
                    Ok(json) => return json,
                    Err(_) => {}
                }
            }
        }
        
        // クエリパラメータをJSONとして出力
        if !entry.request.query_string.is_empty() {
            let mut query_map = HashMap::new();
            for query in &entry.request.query_string {
                query_map.insert(query.name.clone(), query.value.clone());
            }
            
            match serde_json::to_string_pretty(&query_map) {
                Ok(json) => return json,
                Err(_) => {}
            }
        }
        
        "{}".to_string()
    }

    /// レスポンスペイロードを抽出
    /// 
    /// # Arguments
    /// * `entry` - HARエントリ
    /// 
    /// # Returns
    /// * `String` - レスポンスペイロード（JSON形式）
    fn extract_response_payload(&self, entry: &crate::har_types::Entry) -> String {
        if let Some(text) = &entry.response.content.text {
            // Base64デコードが必要な場合
            if let Some(encoding) = &entry.response.content.encoding {
                if encoding == "base64" {
                    match general_purpose::STANDARD.decode(text) {
                        Ok(decoded) => {
                            match String::from_utf8(decoded) {
                                Ok(decoded_text) => {
                                    if self.is_json_content(&entry.response.content.mime_type) {
                                        return self.format_json(&decoded_text);
                                    }
                                    return decoded_text;
                                }
                                Err(_) => return text.clone(),
                            }
                        }
                        Err(_) => return text.clone(),
                    }
                }
            }
            
            // JSONコンテンツの場合はフォーマット
            if self.is_json_content(&entry.response.content.mime_type) {
                return self.format_json(text);
            }
            
            text.clone()
        } else {
            "{}".to_string()
        }
    }

    /// コンテンツタイプがJSONかどうかを判定
    /// 
    /// # Arguments
    /// * `mime_type` - MIMEタイプ
    /// 
    /// # Returns
    /// * `bool` - JSONの場合はtrue
    fn is_json_content(&self, mime_type: &str) -> bool {
        mime_type.contains("application/json") || mime_type.contains("text/json")
    }

    /// JSON文字列をフォーマット
    /// 
    /// # Arguments
    /// * `json_str` - JSON文字列
    /// 
    /// # Returns
    /// * `String` - フォーマットされたJSON文字列
    fn format_json(&self, json_str: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(value) => {
                match serde_json::to_string_pretty(&value) {
                    Ok(formatted) => formatted,
                    Err(_) => json_str.to_string(),
                }
            }
            Err(_) => json_str.to_string(),
        }
    }

    /// URLをUTF-8でデコード
    /// 
    /// # Arguments
    /// * `url_str` - エンコードされたURL文字列
    /// 
    /// # Returns
    /// * `String` - デコードされたURL文字列
    fn decode_url(&self, url_str: &str) -> String {
        // パーセントエンコーディングをデコード
        match urlencoding::decode(url_str) {
            Ok(decoded) => decoded.into_owned(),
            Err(_) => {
                warn!("URLのデコードに失敗しました: {}", url_str);
                url_str.to_string()
            }
        }
    }
}