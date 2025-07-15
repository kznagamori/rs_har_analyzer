//! Excelファイル出力処理を実装

use crate::har_types::AnalysisResult;
use anyhow::{anyhow, Result};
use log::info;
use rust_xlsxwriter::*;

/// Excelエクスポータ
pub struct ExcelExporter;

impl ExcelExporter {
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
        
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        
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
            worksheet.write_string_with_format(row_index, 5, &result.request_url, &cell_format)?;
            worksheet.write_string_with_format(row_index, 6, &result.request_payload, &json_format)?;
            worksheet.write_string_with_format(row_index, 7, &result.response_payload, &json_format)?;
        }
        
        // 列幅の自動調整
        Self::auto_fit_columns(worksheet, results)?;
        
        // ファイルを保存
        workbook.save(output_path)
            .map_err(|e| anyhow!("Excelファイルの保存に失敗しました: {}", e))?;
        
        info!("Excelファイルの出力が完了しました: {}", output_path);
        Ok(())
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