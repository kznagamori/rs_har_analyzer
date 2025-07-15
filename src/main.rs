//! HARファイル解析アプリケーションのメイン関数

mod analyzer;
mod excel_exporter;
mod har_types;
mod logger;

use analyzer::HarAnalyzer;
use anyhow::Result;
use clap::{Arg, Command};
use excel_exporter::ExcelExporter;
use log::{error, info};
use std::path::Path;

/// アプリケーションの設定
#[derive(Debug)]
struct AppConfig {
    input_file: String,
    output_file: String,
    verbose: bool,
}

impl AppConfig {
    /// コマンドライン引数から設定を作成
    /// 
    /// # Returns
    /// * `AppConfig` - アプリケーション設定
    fn from_args() -> Self {
        let matches = Command::new("rs_har_analyzer")
            .version("0.1.0")
            .author("Your Name <your.email@example.com>")
            .about("HARファイルを解析してExcelファイルに出力するツール")
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .value_name("FILE")
                    .help("入力するHARファイルのパス")
                    .required(true)
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("出力するExcelファイルのパス")
                    .default_value("har_analysis.xlsx")
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("詳細ログを出力")
                    .action(clap::ArgAction::SetTrue)
            )
            .get_matches();

        AppConfig {
            input_file: matches.get_one::<String>("input").unwrap().clone(),
            output_file: matches.get_one::<String>("output").unwrap().clone(),
            verbose: matches.get_flag("verbose"),
        }
    }

    /// 設定の妥当性を検証
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk、失敗時はエラー
    fn validate(&self) -> Result<()> {
        // 入力ファイルの存在確認
        if !Path::new(&self.input_file).exists() {
            return Err(anyhow::anyhow!("入力ファイルが見つかりません: {}", self.input_file));
        }

        // 入力ファイルの拡張子確認
        if !self.input_file.to_lowercase().ends_with(".har") {
            return Err(anyhow::anyhow!("入力ファイルはHARファイル(.har)である必要があります"));
        }

        // 出力ファイルの拡張子確認
        if !self.output_file.to_lowercase().ends_with(".xlsx") {
            return Err(anyhow::anyhow!("出力ファイルはExcelファイル(.xlsx)である必要があります"));
        }

        Ok(())
    }
}

/// アプリケーションを実行
/// 
/// # Arguments
/// * `config` - アプリケーション設定
/// 
/// # Returns
/// * `Result<()>` - 成功時はOk、失敗時はエラー
async fn run_app(config: AppConfig) -> Result<()> {
    info!("HARファイル解析を開始します");
    info!("入力ファイル: {}", config.input_file);
    info!("出力ファイル: {}", config.output_file);

    // HARファイルの解析
    let analyzer = HarAnalyzer::new(&config.input_file)?;
    let results = analyzer.analyze()?;

    if results.is_empty() {
        info!("解析対象のGET/POSTリクエストが見つかりませんでした");
        return Ok(());
    }

    // 解析結果のサマリーを出力
    info!("解析結果のサマリー:");
    info!("  - 総エントリ数: {}", results.len());
    
    let get_count = results.iter().filter(|r| r.method == "GET").count();
    let post_count = results.iter().filter(|r| r.method == "POST").count();
    
    info!("  - GETリクエスト: {}", get_count);
    info!("  - POSTリクエスト: {}", post_count);

    // ステータスコード別の集計
    let mut status_counts = std::collections::HashMap::new();
    for result in &results {
        *status_counts.entry(result.status_code).or_insert(0) += 1;
    }

    info!("  - ステータスコード別集計:");
    for (status, count) in status_counts.iter() {
        info!("    {}: {}", status, count);
    }

    // Excelファイルに出力
    ExcelExporter::export(&results, &config.output_file)?;

    info!("HARファイル解析が完了しました");
    Ok(())
}

/// メイン関数
/// 
/// # Returns
/// * `Result<()>` - 成功時はOk、失敗時はエラー
#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_args();
    
    // ログシステムの初期化
    logger::init_logger(config.verbose)?;

    // 設定の妥当性検証
    if let Err(e) = config.validate() {
        error!("設定エラー: {}", e);
        std::process::exit(1);
    }

    // アプリケーションの実行
    if let Err(e) = run_app(config).await {
        error!("実行エラー: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
