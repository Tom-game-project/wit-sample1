use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use component_features::shift_calendar_manager::{
    ShiftCalendarManager, 
    WeekStatus
};

// 引数を構造体として定義します
#[derive(Parser)]
#[command(name = "shift_tools")]
#[command(version = "0.1.0")]
#[command(about = "wasm-shift-managerに関わるデータの操作をします", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 指定したinit_deltaに変更します
    ChangeDelta {
        /// カレンダーデータファイル
        file: PathBuf,

        /// init_deltaを変更する
        #[arg(short, long)]
        init_delta: usize,

        #[arg(short, long)]
        out: Option<PathBuf>
    }
}

fn change_delta(file: PathBuf, init_delta: usize, out: Option<PathBuf>) {
    // 'count' コマンドが選ばれたときの処理
    match fs::read_to_string(&file) {
        Ok(text) => {
            if let Ok(shift_calendar_manager) = serde_json::from_str::<ShiftCalendarManager>(&text) {
                let mut timeline = Vec::new();
                let mut counter = init_delta;
                for week_status in shift_calendar_manager.timeline{
                    if let WeekStatus::Active { logical_delta:_ } = week_status {
                        timeline.push(WeekStatus::Active { logical_delta:counter });
                        counter += 1;
                    } else {
                        timeline.push(WeekStatus::Skipped);
                    }
                }

                let return_shift_manager_data = ShiftCalendarManager {
                    base_abs_week: shift_calendar_manager.base_abs_week,
                    initial_delta: init_delta,
                    timeline
                };

                if let Some(path) = out {
                    if let Err(_e) = fs::write(
                        path,
                        serde_json::to_string(&return_shift_manager_data).unwrap()
                    ) {
                        eprintln!("ファイルの書き込みに失敗しました");
                    }
                } else {
                    println!("{:?}", return_shift_manager_data);
                }
            } else {
                eprintln!("ファイルが形式に沿っていません");
            }
        }
        Err(e) => {
            eprintln!("エラー: ファイル '{}' を読めませんでした: {}", file.display(), e);
        }
    }
}

fn main() {
    let args = Cli::parse();

    // 3. パターンマッチで分岐処理
    match args.command {
        Commands::ChangeDelta { file, init_delta, out } => {
            change_delta(file, init_delta, out);
        }
    }
}
