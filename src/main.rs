use std::env;
use std::path::Path;
use std::time::{Duration, Instant};
use event_chains::{EventChain, EventContext, FaultToleranceMode};
use image_upscaling::algorithms::image::Image;
use image_upscaling::algorithms::prelude::*;
use image_upscaling::algorithms::slow::IterativeBackProjection;
use image_upscaling::content_analysis::ContentAnalysis;
use image_upscaling::event_chain_pipeline::analyze_content_event::AnalyzeContentEvent;
use image_upscaling::event_chain_pipeline::detect_quality_issues_event::DetectQualityIssuesEvent;
use image_upscaling::event_chain_pipeline::load_image_event::LoadImageEvent;
use image_upscaling::event_chain_pipeline::pipeline_config::PipelineConfig;
use image_upscaling::event_chain_pipeline::postprocess_image_event::PostProcessImageEvent;
use image_upscaling::event_chain_pipeline::preprocess_image_event::PreprocessImageEvent;
use image_upscaling::event_chain_pipeline::save_image_event::SaveImageEvent;
use image_upscaling::event_chain_pipeline::upscale_with_strategy_event::UpscaleWithStrategyEvent;
use image_upscaling::event_chain_pipeline::validate_image_event::ValidateImageEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessingMode {
    Pipeline,      // Multi-step intelligent pipeline
    Traditional,   // Direct upscaling (NO event chains)
    Compare,       // Compare both approaches
}

fn print_usage() {
    println!("|--------------------------------------------------------------|");
    println!("|               Image Upscaler CLI v2.0                        |");
    println!("|             (Intelligent Pipeline Edition)                   |");
    println!("|--------------------------------------------------------------|");
    println!();

    println!("Usage:");
    println!("  cargo run <input> <output> [scale] [options]");
    println!();

    println!("Arguments:");
    println!("  <input>      Path to input image (PNG, JPG, etc.)");
    println!("  <output>     Path to output image");
    println!("  [scale]      Scale factor (default: 2.0)");
    println!();

    println!("Processing Modes:");
    println!("  --mode=pipeline     Intelligent multi-step pipeline (NEW, default)");
    println!("  --mode=traditional  Direct upscaling");
    println!("  --mode=compare      Compare both approaches");
    println!();

    println!("Algorithm Selection:");
    println!("  [No --algorithm]    Pipeline: Auto-detect (recommended)");
    println!("                      Others: Use lanczos3 (default)");
    println!("  --algorithm=NAME    Force specific algorithm (all modes)");
    println!();

    println!("Pipeline-Only Options:");
    println!("  --no-preprocess     Disable preprocessing");
    println!("  --no-postprocess    Disable post-processing");
    println!();

    println!("Available Algorithms:");
    println!("  nearest      Nearest Neighbor (fastest, pixel-perfect)");
    println!("  bilinear     Bilinear Interpolation (fast, smooth)");
    println!("  bicubic      Bicubic Interpolation (balanced)");
    println!("  lanczos2     Lanczos2 (sharp, fast)");
    println!("  lanczos3     Lanczos3 (sharpest, recommended)");
    println!("  lanczos4     Lanczos4 (maximum quality)");
    println!("  ibp-fast     Iterative Back-Projection Fast (5 iterations)");
    println!("  ibp          Iterative Back-Projection Standard (10 iterations)");
    println!("  ibp-quality  Iterative Back-Projection Quality (20 iterations)");
    println!();

    println!("Examples:");
    println!("  # Pipeline with auto-selection (recommended)");
    println!("  cargo run input.jpg output.png");
    println!();
    println!("  # Pipeline with manual algorithm choice");
    println!("  cargo run input.jpg output.png --algorithm=lanczos3");
    println!();
    println!("  # Traditional (no event chains at all)");
    println!("  cargo run input.jpg output.png --mode=traditional --algorithm=bicubic");
    println!();
    println!("  # Compare both modes");
    println!("  cargo run input.jpg output.png --mode=compare");
    println!();
}

fn get_traditional_upscaler(algorithm: &str) -> Result<Box<dyn image_upscaling::algorithms::upscaler::Upscaler>, String> {
    match algorithm.to_lowercase().as_str() {
        "nearest" => Ok(Box::new(NearestNeighbor)),
        "bilinear" => Ok(Box::new(Bilinear)),
        "bicubic" => Ok(Box::new(Bicubic)),
        "lanczos2" => Ok(Box::new(Lanczos::fast())),
        "lanczos3" => Ok(Box::new(Lanczos::new())),
        "lanczos4" => Ok(Box::new(Lanczos::high_quality())),
        "ibp-fast" => Ok(Box::new(IterativeBackProjection::fast())),
        "ibp" | "ibp-standard" => Ok(Box::new(IterativeBackProjection::new())),
        "ibp-quality" => Ok(Box::new(IterativeBackProjection::quality())),
        _ => Err(format!("Unknown algorithm: {}", algorithm)),
    }
}

fn process_with_pipeline(
    input_path: &str,
    output_path: &str,
    scale_factor: f32,
    force_algorithm: Option<String>,
    enable_preprocessing: bool,
    enable_postprocessing: bool,
) -> Result<(Image, Duration), String> {
    println!();
    println!("Building intelligent pipeline...");

    let metrics = event_chains::middleware::metrics::MetricsMiddleware::new();
    let metrics_clone = metrics.clone();

    // Create pipeline configuration
    let mut config = PipelineConfig::new(scale_factor);
    if let Some(algo) = force_algorithm {
        config = config.with_algorithm(algo);
    }
    config = config.with_preprocessing(enable_preprocessing);
    config = config.with_postprocessing(enable_postprocessing);

    // Build multi-phase pipeline
    let pipeline = EventChain::new()
        // Infrastructure middleware
        .middleware(metrics)
        .middleware(event_chains::middleware::timing::TimingMiddleware::new())
        .middleware(event_chains::middleware::logging::LoggingMiddleware::info())

        // Phase 1: Load & Validate (Fast Failures)
        .event(LoadImageEvent::from_path(input_path))
        .event(ValidateImageEvent::new())

        // Phase 2: Analysis & Planning
        .event(AnalyzeContentEvent::new())
        .event(DetectQualityIssuesEvent::new())

        // Phase 3: Conditional Preprocessing
        .event(PreprocessImageEvent::new())

        // Phase 4: Smart Upscaling
        .event(UpscaleWithStrategyEvent::new())

        // Phase 5: Post-processing
        .event(PostProcessImageEvent::new())

        // Phase 6: Output
        .event(SaveImageEvent::to_path(output_path))

        .with_fault_tolerance(FaultToleranceMode::BestEffort);

    println!("   Pipeline configured with 6 phases");
    println!("   Middleware: Metrics, Timing, Logging");
    println!("   Fault tolerance: BestEffort");
    println!();

    // Execute pipeline
    println!("Executing pipeline...");
    println!();

    let mut context = EventContext::new();
    context.set("config", config);

    let start = Instant::now();
    let result = pipeline.execute(&mut context);
    let duration = start.elapsed();

    println!();

    // Check result
    if !result.success {
        eprintln!("Pipeline failed:");
        for failure in &result.failures {
            eprintln!("   - {}: {}", failure.event_name, failure.error_message);
        }
        return Err("Pipeline execution failed".to_string());
    }

    println!("Pipeline completed successfully!");

    // Get output image
    let output_image: Image = match context.get("output_image") {
        Some(img) => img,
        None => return Err("No output image in context".to_string()),
    };

    println!();
    println!("Pipeline Results:");
    println!("   Output size:   {}x{}", output_image.width, output_image.height);
    println!("   Total pixels:  {}", output_image.pixels.len());
    println!("   Duration:      {:.3}s", duration.as_secs_f64());

    // Print algorithm used
    if let Some(algo) = context.get::<String>("algorithm_used") {
        println!("   Algorithm:     {}", algo);
    }

    // Print metrics
    println!();
    println!("Performance Metrics:");
    metrics_clone.print_summary();

    Ok((output_image, duration))
}

fn process_traditional(
    input_path: &str,
    output_path: &str,
    algorithm_name: &str,
    scale_factor: f32,
) -> Result<(Image, Duration), String> {
    println!();
    println!("Traditional mode (direct processing)...");
    println!("   Algorithm: {}", algorithm_name);
    println!("   Scale:     {}x", scale_factor);
    println!();

    // Load image
    println!("Loading image...");
    let start_load = Instant::now();
    let image = Image::load(input_path)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    let load_duration = start_load.elapsed();
    println!("   Loaded {}x{} in {:.3}s", image.width, image.height, load_duration.as_secs_f64());

    // Get upscaler
    let upscaler = get_traditional_upscaler(algorithm_name)?;

    // Upscale
    println!();
    println!("Upscaling...");
    let start_upscale = Instant::now();
    let output_image = upscaler.upscale(&image, scale_factor);
    let upscale_duration = start_upscale.elapsed();
    println!("   Upscaled to {}x{} in {:.3}s",
             output_image.width, output_image.height, upscale_duration.as_secs_f64());

    // Save
    println!();
    println!("Saving image...");
    let start_save = Instant::now();
    output_image.save(output_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;
    let save_duration = start_save.elapsed();
    println!("   Saved in {:.3}s", save_duration.as_secs_f64());

    let total_duration = load_duration + upscale_duration + save_duration;

    println!();
    println!("Processing complete!");
    println!("   Total time: {:.3}s", total_duration.as_secs_f64());

    Ok((output_image, total_duration))
}

fn compare_modes(
    input_path: &str,
    output_path: &str,
    scale_factor: f32,
    force_algorithm: Option<String>,
) -> Result<(), String> {
    println!();
    println!("===============================================================");
    println!("                    COMPARISON MODE                            ");
    println!("===============================================================");
    println!();

    // First, analyze the image to determine the best algorithm
    let image = Image::load(input_path)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    let analysis = ContentAnalysis::analyze(&image);
    println!("Content Analysis:");
    analysis.print_summary();

    // Use forced algorithm if provided, otherwise use recommendation
    let algorithm_to_use = force_algorithm.clone()
        .unwrap_or_else(|| analysis.content_type.recommended_algorithm().to_string());

    println!();
    println!("---------------------------------------------------------------");
    println!();

    // Run traditional mode
    println!("TEST 1: Traditional Mode (NO event chains)");
    println!("---------------------------------------------------------------");

    let trad_output = format!("{}_traditional.png",
                              Path::new(output_path).file_stem().unwrap().to_str().unwrap());

    let trad_result = process_traditional(
        input_path,
        &trad_output,
        &algorithm_to_use,
        scale_factor,
    );

    let trad_duration = match trad_result {
        Ok((_, dur)) => Some(dur),
        Err(e) => {
            println!();
            println!("Traditional mode skipped: {}", e);
            println!();
            None
        }
    };

    println!();
    println!("---------------------------------------------------------------");
    println!();


    // Run pipeline mode
    println!("TEST 2: Pipeline Mode                     ");
    println!("---------------------------------------------------------------");

    let pipe_output = format!("{}_pipeline.png",
                              Path::new(output_path).file_stem().unwrap().to_str().unwrap());

    let pipe_result = process_with_pipeline(
        input_path,
        &pipe_output,
        scale_factor,
        force_algorithm.clone(),  // Pass the forced algorithm!
        true,
        true,
    );

    let pipe_duration = match pipe_result {
        Ok((_, dur)) => Some(dur),
        Err(e) => {
            println!();
            println!("Pipeline mode skipped: {}", e);
            println!();
            None
        }
    };

    // Comparison summary - only if we have at least one result
    if trad_duration.is_none() && pipe_duration.is_none() {
        return Err("All modes failed or were skipped".to_string());
    }

    println!();
    println!("===============================================================");
    println!("                    COMPARISON SUMMARY                         ");
    println!("===============================================================");
    println!();

    println!("|------------------------+--------------+--------------+------------|");
    println!("| Mode                   | Duration     | Overhead     | Output     |");
    println!("|------------------------+--------------+--------------+------------|");

    // Use first available duration as baseline
    let baseline = trad_duration
        .or(pipe_duration)
        .unwrap();

    if let Some(trad_dur) = trad_duration {
        println!("| Traditional            | {:>9.3}s | baseline     | {}       |",
                 trad_dur.as_secs_f64(), Path::new(&trad_output).file_name().unwrap().to_str().unwrap());
    } else {
        println!("| Traditional            | SKIPPED      | ---          | N/A        |");
    }

    if let Some(pipe_dur) = pipe_duration {
        let overhead = (pipe_dur.as_secs_f64() / baseline.as_secs_f64() - 1.0) * 100.0;
        let overhead_str = if overhead > 0.0 {
            format!("+{:.1}%", overhead)
        } else {
            format!("{:.1}%", overhead)
        };
        println!("| Pipeline          | {:>9.3}s | {:>12} | {}       |",
                 pipe_dur.as_secs_f64(), overhead_str, Path::new(&pipe_output).file_name().unwrap().to_str().unwrap());
    } else {
        println!("| Pipeline          | SKIPPED      | ---          | N/A        |");
    }

    println!("|------------------------+--------------+--------------+------------|");
    println!();

    println!("Key Insights:");
    if let Some(trad_dur) = trad_duration {
        println!("  * Traditional: Baseline (no overhead, no features)");

        if let Some(pipe_dur) = pipe_duration {
            let pipe_overhead = (pipe_dur.as_secs_f64() / trad_dur.as_secs_f64() - 1.0) * 100.0;
            println!("  * Pipeline : {:.1}% overhead, smart features + observability", pipe_overhead);

            if let Some(ec_dur) = pipe_duration {
                let improvement = (ec_dur.as_secs_f64() - pipe_dur.as_secs_f64()) / trad_dur.as_secs_f64() * 100.0;
                if improvement > 0.0 {
                    println!();
                    println!("RESULT: Pipeline is {:.1}% faster than old event chains!", improvement.abs());
                }
            }
        }
    } else {
        println!("  * Traditional: SKIPPED (algorithm not available)");
    }

    println!();

    if pipe_duration.is_some() {
        if let Some(trad_dur) = trad_duration {
            if let Some(pipe_dur) = pipe_duration {
                let pipe_overhead = (pipe_dur.as_secs_f64() / trad_dur.as_secs_f64() - 1.0) * 100.0;

                if pipe_overhead < 0.0 {
                    println!("RESULT: Pipeline demonstrates REAL event chains value!");
                    println!("        ({:.1}% FASTER than baseline)", pipe_overhead.abs());
                } else if pipe_overhead < 2.0 {
                    println!("RESULT: Pipeline overhead is minimal ({:.1}%) for significant gains!", pipe_overhead);
                } else {
                    println!("RESULT: Pipeline overhead ({:.1}%) buys intelligent processing", pipe_overhead);
                }
            }
        } else {
            println!("RESULT: Pipeline provides intelligent processing with observability");
        }
    }

    println!();
    println!("===============================================================");
    println!();

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Parse options
    let mut scale_factor = 2.0f32;
    let mut mode = ProcessingMode::Pipeline;
    let mut force_algorithm: Option<String> = None;
    let mut enable_preprocessing = true;
    let mut enable_postprocessing = true;

    for arg in args.iter().skip(3) {
        if arg.starts_with("--mode=") {
            let mode_str = &arg[7..];
            mode = match mode_str.to_lowercase().as_str() {
                "pipeline" => ProcessingMode::Pipeline,
                "traditional" | "direct" => ProcessingMode::Traditional,
                "compare" | "comparison" => ProcessingMode::Compare,
                _ => {
                    eprintln!("Error: Unknown mode '{}'. Use 'pipeline', 'traditional', or 'compare'", mode_str);
                    std::process::exit(1);
                }
            };
        } else if arg.starts_with("--algorithm=") {
            force_algorithm = Some(arg[12..].to_string());
        } else if arg == "--no-preprocess" {
            enable_preprocessing = false;
        } else if arg == "--no-postprocess" {
            enable_postprocessing = false;
        } else if !arg.starts_with("--") {
            if let Ok(val) = arg.parse::<f32>() {
                scale_factor = val;
            }
        }
    }

    // Validate scale factor
    if scale_factor <= 0.0 || scale_factor > 100.0 {
        eprintln!("Error: Scale factor must be between 0 and 100");
        std::process::exit(1);
    }

    println!();
    println!("|---------------------------------------------------------------|");
    println!("|              Image Upscaling                                  |");
    println!("|---------------------------------------------------------------|");
    println!();

    println!("Configuration:");
    println!("   Input:      {}", input_path);
    println!("   Output:     {}", output_path);
    println!("   Scale:      {}x", scale_factor);
    println!("   Mode:       {:?}", mode);
    if let Some(ref algo) = force_algorithm {
        println!("   Algorithm:  {} (forced)", algo);
    }

    // For non-pipeline modes, default to lanczos3 if no algorithm specified
    let default_algo = force_algorithm.clone().unwrap_or_else(|| {
        if mode != ProcessingMode::Pipeline {
            "lanczos3".to_string()
        } else {
            String::new()
        }
    });

    // Execute based on mode
    let result = match mode {
        ProcessingMode::Pipeline => {
            process_with_pipeline(
                input_path,
                output_path,
                scale_factor,
                force_algorithm,
                enable_preprocessing,
                enable_postprocessing,
            ).map(|_| ())
        }
        ProcessingMode::Traditional => {
            process_traditional(
                input_path,
                output_path,
                &default_algo,
                scale_factor,
            ).map(|_| ())
        }
        ProcessingMode::Compare => {
            compare_modes(input_path, output_path, scale_factor, force_algorithm)
        }
    };

    match result {
        Ok(_) => {
            println!();
            println!("Done!");
            println!();
        }
        Err(e) => {
            eprintln!();
            eprintln!("Error: {}", e);
            eprintln!();
            std::process::exit(1);
        }
    }
}
