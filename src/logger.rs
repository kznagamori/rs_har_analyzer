//! ログ設定を実装

use anyhow::Result;
use chrono::Utc;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

/// ログシステムを初期化
/// 
/// # Arguments
/// * `verbose` - 詳細ログを出力するかどうか
/// 
/// # Returns
/// * `Result<()>` - 成功時はOk、失敗時はエラー
pub fn init_logger(verbose: bool) -> Result<()> {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
