use std::path::PathBuf;

fn main() {
    #[cfg(not(has_proprietary_ai))]
    {
        eprintln!("eval_models requires the proprietary AI kernel (src/proprietary/).");
        std::process::exit(1);
    }

    #[cfg(has_proprietary_ai)]
    {
        let args: Vec<String> = std::env::args().collect();
        let mut manifest: Option<PathBuf> = None;
        let mut is_bench = false;
        let mut model_id = String::from("yunet");
        let mut ep_mode = String::from("openvino_npu");
        let mut batch_size = 1usize;
        let mut custom_h = 640usize;
        let mut custom_w = 640usize;
        let mut warmup = 3usize;
        let mut iterations = 15usize;

        let mut idx = 1;
        while idx < args.len() {
            match args[idx].as_str() {
                "--bench" => {
                    is_bench = true;
                    idx += 1;
                }
                "--model" if idx + 1 < args.len() => {
                    model_id = args[idx + 1].clone();
                    idx += 2;
                }
                "--ep" if idx + 1 < args.len() => {
                    ep_mode = args[idx + 1].clone();
                    idx += 2;
                }
                "--batch" if idx + 1 < args.len() => {
                    batch_size = args[idx + 1].parse().unwrap_or(1);
                    idx += 2;
                }
                "--h" if idx + 1 < args.len() => {
                    custom_h = args[idx + 1].parse().unwrap_or(640);
                    idx += 2;
                }
                "--w" if idx + 1 < args.len() => {
                    custom_w = args[idx + 1].parse().unwrap_or(640);
                    idx += 2;
                }
                "--warmup" if idx + 1 < args.len() => {
                    warmup = args[idx + 1].parse().unwrap_or(3);
                    idx += 2;
                }
                "--iters" if idx + 1 < args.len() => {
                    iterations = args[idx + 1].parse().unwrap_or(15);
                    idx += 2;
                }
                "--manifest" if idx + 1 < args.len() => {
                    manifest = Some(PathBuf::from(&args[idx + 1]));
                    idx += 2;
                }
                _ => {
                    idx += 1;
                }
            }
        }

        if is_bench {
            match photoselect_desktop_lib::npu_engine::NpuEngine::run_onnx_bench(
                &model_id,
                &ep_mode,
                batch_size,
                custom_h,
                custom_w,
                warmup,
                iterations,
            ) {
                Ok(res) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&res).unwrap_or_else(|e| format!("{e}"))
                    );
                }
                Err(err) => {
                    eprintln!("ONNX bench failed: {}", err);
                    std::process::exit(1);
                }
            }
            return;
        }

        match photoselect_desktop_lib::npu_engine::run_eval_cli(manifest) {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|e| format!("{e}"))
                );
            }
            Err(err) => {
                eprintln!("Evaluation failed: {}", err);
                std::process::exit(1);
            }
        }
    }
}
