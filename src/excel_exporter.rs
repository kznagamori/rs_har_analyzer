//! Excelファイル出力処理を実装

use crate::har_types::AnalysisResult;
use anyhow::{anyhow, Result};
use log::{info, warn};
use rust_xlsxwriter::*;
use std::fs;
use std::path::Path;

/// Excelエクスポータ
pub struct ExcelExporter;

impl ExcelExporter {
    /// 行番号と列番号をExcel形式のセル参照に変換
    /// 
    /// # Arguments
    /// * `row` - 行番号（1から開始）
    /// * `col` - 列番号（0から開始）
    /// 
    /// # Returns
    /// * `String` - Excel形式のセル参照（例: "A1", "B2", "AA10"）
    fn to_excel_cell_reference(row: u32, col: u16) -> String {
        let mut column_name = String::new();
        let mut col_num = col as u32 + 1; // 1から開始に変換
        
        while col_num > 0 {
            col_num -= 1;
            column_name.insert(0, (b'A' + (col_num % 26) as u8) as char);
            col_num /= 26;
        }
        
        format!("{}{}", column_name, row)
    }

    /// 解析結果をExcelファイルに出力
    /// 
    /// # Arguments
    /// * `results` - 解析結果のリスト
    /// * `output_path` - 出力ファイルのパス
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk、失敗時はエラー
    pub fn export(results: &[AnalysisResult], output_path: &str) -> Result<()> {
        info!("Excelファイルに出力しています: {}", output_path);
        
        // 出力ディレクトリを作成（必要に応じて）
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("出力ディレクトリの作成に失敗しました: {}", e))?;
        }

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        
        // ベースファイル名を取得（拡張子なし）
        let base_name = Path::new(output_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        
        let output_dir = Path::new(output_path)
            .parent()
            .unwrap_or(Path::new("."));
        
        // ヘッダー行の設定
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xD3D3D3))
            .set_border(FormatBorder::Thin);
        
        let headers = vec![
            "時刻",
            "送信元IP",
            "送信先IP",
            "メソッド",
            "ステータスコード",
            "リクエストURL",
            "リクエストペイロード",
            "レスポンスペイロード",
        ];
        
        // ヘッダー行を書き込み
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)?;
        }
        
        // データ行のフォーマット
        let cell_format = Format::new()
            .set_border(FormatBorder::Thin)
            .set_text_wrap();
        
        let json_format = Format::new()
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_font_name("Consolas")
            .set_font_size(9);
        
        // データ行を書き込み
        for (row, result) in results.iter().enumerate() {
            let row_index = (row + 1) as u32;
            
            // 各列のデータを書き込み
            worksheet.write_string_with_format(row_index, 0, &result.timestamp, &cell_format)?;
            worksheet.write_string_with_format(row_index, 1, &result.source_ip, &cell_format)?;
            worksheet.write_string_with_format(row_index, 2, &result.destination_ip, &cell_format)?;
            worksheet.write_string_with_format(row_index, 3, &result.method, &cell_format)?;
            worksheet.write_number_with_format(row_index, 4, result.status_code as f64, &cell_format)?;
            
            // URLの処理（長い場合は切り詰め）
            let url_content = Self::handle_large_content(
                &result.request_url, 
                base_name, 
                output_dir, 
                row_index + 1, // ヘッダー行を考慮
                5
            )?;
            worksheet.write_string_with_format(row_index, 5, &url_content, &cell_format)?;
            
            // リクエストペイロードの処理
            let request_content = Self::handle_large_content(
                &result.request_payload, 
                base_name, 
                output_dir, 
                row_index + 1, // ヘッダー行を考慮
                6
            )?;
            worksheet.write_string_with_format(row_index, 6, &request_content, &json_format)?;
            
            // レスポンスペイロードの処理
            let response_content = Self::handle_large_content(
                &result.response_payload, 
                base_name, 
                output_dir, 
                row_index + 1, // ヘッダー行を考慮
                7
            )?;
            worksheet.write_string_with_format(row_index, 7, &response_content, &json_format)?;
        }
        
        // 列幅の自動調整
        Self::auto_fit_columns(worksheet, results)?;
        
        // ファイルを保存
        workbook.save(output_path)
            .map_err(|e| anyhow!("Excelファイルの保存に失敗しました: {}", e))?;
        
        info!("Excelファイルの出力が完了しました: {}", output_path);
        Ok(())
    }

    /// 大きなコンテンツを処理（必要に応じて外部ファイルに保存）
    /// 
    /// # Arguments
    /// * `content` - 処理するコンテンツ
    /// * `base_name` - ベースファイル名
    /// * `output_dir` - 出力ディレクトリ
    /// * `row` - 行番号
    /// * `col` - 列番号
    /// 
    /// # Returns
    /// * `Result<String>` - セルに入れる文字列
    fn handle_large_content(
        content: &str, 
        base_name: &str, 
        output_dir: &Path, 
        row: u32, 
        col: u16
    ) -> Result<String> {
        const EXCEL_LIMIT: usize = 32000; // 安全マージンを考慮
        
        if content.len() <= EXCEL_LIMIT {
            Ok(content.to_string())
        } else {
            // 外部ファイルに保存
            let cell_ref = Self::to_excel_cell_reference(row, col);
            let filename = format!("{}_{}.txt", base_name, cell_ref);
            let filepath = output_dir.join(&filename);
            
            fs::write(&filepath, content)
                .map_err(|e| anyhow!("外部ファイルの書き込みに失敗しました: {}", e))?;
            
            warn!("大きなコンテンツを外部ファイルに保存しました: {}", filename);
            
            // セルには参照情報を保存
            Ok(format!("ファイル参照: {} ({}文字)", filename, content.len()))
        }
    }

    /// 列幅を自動調整
    /// 
    /// # Arguments
    /// * `worksheet` - ワークシート
    /// * `results` - 解析結果のリスト
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時はOk、失敗時はエラー
    fn auto_fit_columns(
        worksheet: &mut Worksheet,
        results: &[AnalysisResult],
    ) -> Result<()> {
        // 各列の適切な幅を計算
        let column_widths = vec![
            20.0, // 時刻
            15.0, // 送信元IP
            20.0, // 送信先IP
            10.0, // メソッド
            15.0, // ステータスコード
            50.0, // リクエストURL
            30.0, // リクエストペイロード
            30.0, // レスポンスペイロード
        ];
        
        // 列幅を設定
        for (col, width) in column_widths.iter().enumerate() {
            worksheet.set_column_width(col as u16, *width)?;
        }
        
        // 行の高さを設定（JSONペイロードの表示のため）
        for row in 1..=results.len() {
            worksheet.set_row_height(row as u32, 60.0)?;
        }
        
        Ok(())
    }
}