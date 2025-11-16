# 🎉 CLI Tool Complete - Full Image Processing Pipeline!

## What We Just Added

A **production-ready CLI tool** for upscaling images using event_chains, with full I/O support for real image files
### Supports ###
- PNG
- JPEG
- GIF
- BMP
- TIFF
- WEBP

### 2. CLI Tool
```bash
cargo run --release input.jpg output.png lanczos3 2.0
```

## Usage Examples

### Basic Usage

**Use recommended algorithm (lanczos3)**
```bash
cargo run --example upscale_cli --release photo.jpg photo_2x.jpg lanczos3
```
**compare results between traditional implementation and event chains implementation**`
```bash
 cargo run -- "C:\Users\jglov\Pictures\200x200Avatar.png" newOutput.png 15.0  --mode=compare
```
**run with pipeline mode specifically with recommended algorithm**
```bash
 cargo run -- "C:\Users\jglov\Pictures\200x200Avatar.png" output.png 15.0 --mode=pipeline
```

**run with traditional mode specifically with recommended algorithm**
```bash
 cargo run -- "C:\Users\jglov\Pictures\200x200Avatar.png" output.png 15.0 --mode=traditional
```

**run with pipeline mode and force the selection of the ibp-quality algorithm**
```bash
 cargo run -- "C:\Users\jglov\Pictures\200x200Avatar.png" output.png 15.0 --algorithm=ibp-quality
```


## Sample Output

```txt
|---------------------------------------------------------------|
|              Image Upscaling                                  |
|---------------------------------------------------------------|

Configuration:
   Input:      C:\Users\jglov\Pictures\200x200Avatar.png
   Output:     output.png
   Scale:      15x
   Mode:       Compare

===============================================================
                    COMPARISON MODE                            
===============================================================

Content Analysis:
   Content Analysis:
     Type:              Artwork
     Unique colors:     1671
     Edge sharpness:    0.17
     Gradient smooth:   0.92
     Text likelihood:   0.06
     Noise level:       0.01
     Recommended algo:  lanczos3

---------------------------------------------------------------

TEST 1: Traditional Mode (NO event chains)
---------------------------------------------------------------

Traditional mode (direct processing)...
   Algorithm: lanczos3
   Scale:     15x

Loading image...
   Loaded 200x200 in 0.006s

Upscaling...
   Upscaled to 3000x3000 in 27.789s

Saving image...
   Saved in 3.348s

Processing complete!
   Total time: 31.143s

---------------------------------------------------------------

TEST 2: Pipeline Mode                     
---------------------------------------------------------------

Building intelligent pipeline...
   Pipeline configured with 6 phases
   Middleware: Metrics, Timing, Logging
   Fault tolerance: BestEffort

Executing pipeline...

   Loaded 200x200 image
  LoadImage took 6.62ms
[INFO]  Completed event: LoadImage
   Validation passed
  ValidateImage took 96µs
[INFO]  Completed event: ValidateImage
   Content Type: Artwork
   Recommended Algorithm: lanczos3
  AnalyzeContent took 17.46ms
[INFO]  Completed event: AnalyzeContent
   Quality Issues:
     - Low edge sharpness detected
  DetectQualityIssues took 105µs
[INFO]  Completed event: DetectQualityIssues
   Applying sharpening...
  PreprocessImage took 8.47ms
[INFO]  Completed event: PreprocessImage
   Auto-selected: lanczos3 (based on Artwork)
   Upscaling with Lanczos3 (15x)...
   Output size: 3000x3000
  UpscaleWithStrategy took 27.67s
[INFO]  Completed event: UpscaleWithStrategy
   Post-processing complete
  PostProcessImage took 43µs
[INFO]  Completed event: PostProcessImage
   Image saved successfully
  SaveImage took 3.32s
[INFO]  Completed event: SaveImage

Pipeline completed successfully!

Pipeline Results:
   Output size:   3000x3000
   Total pixels:  9000000
   Duration:      31.016s
   Algorithm:     lanczos3

Performance Metrics:

=== Event Metrics Summary ===
Event                          Total    Success     Failed     Avg (µs)     Min (µs)     Max (µs)  Success %
-------------------------------------------------------------------------------------------------------------------
AnalyzeContent                     1          1          0        17453        17453        17453     100.0%
DetectQualityIssues                1          1          0          102          102          102     100.0%
LoadImage                          1          1          0         6612         6612         6612     100.0%
PostProcessImage                   1          1          0           42           42           42     100.0%
PreprocessImage                    1          1          0         8472         8472         8472     100.0%
SaveImage                          1          1          0      3316254      3316254      3316254     100.0%
UpscaleWithStrategy                1          1          0     27666293     27666293     27666293     100.0%
ValidateImage                      1          1          0           93           93           93     100.0%


===============================================================
                    COMPARISON SUMMARY                         
===============================================================

|------------------------+--------------+--------------+------------|
| Mode                   | Duration     | Overhead     | Output     |
|------------------------+--------------+--------------+------------|
| Traditional            |    31.143s   | baseline     | output_traditional.png       |
| Pipeline               |    31.016s   |   -0.4%      | output_pipeline.png       |
|------------------------+--------------+--------------+------------|

Key Insights:
  * Traditional: Baseline (no overhead, no features)
  * Pipeline : -0.4% overhead, smart features + observability

RESULT: Pipeline demonstrates REAL event chains value!
        (0.4% FASTER than baseline)

===============================================================


Done!
```

## Pipeline Architecture

```md
┌─────────────────────────────────────────────────────────┐
│                   Event Chain Pipeline                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Middleware Stack (LIFO)                                │
│  ├── MetricsMiddleware    (Outermost)                   │
│  ├── TimingMiddleware                                   │
│  └── LoggingMiddleware    (Innermost)                   │
│                                                         │
│  Event Pipeline (FIFO)                                  │
│  ┌────────────────────────────────────────────────┐     │
│  │ Phase 1: Load & Validate                       │     │
│  │  ├── LoadImageEvent                            │     │
│  │  └── ValidateImageEvent                        │     │
│  └────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────┐     │
│  │ Phase 2: Analysis                              │     │
│  │  ├── AnalyzeContentEvent                       │     │
│  │  └── DetectQualityIssuesEvent                  │     │
│  └────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────┐     │
│  │ Phase 3: Processing                            │     │
│  │  ├── PreprocessImageEvent                      │     │
│  │  ├── UpscaleWithStrategyEvent                  │     │
│  │  └── PostprocessImageEvent                     │     │
│  └────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────┐     │
│  │ Phase 4: Output                                │     │
│  │  └── SaveImageEvent                            │     │
│  └────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

**You get all this for FREE:**
- Automatic timing
- Complete logging
- Metrics collection
- Error handling
- Type-safe data passing

**This is what production-ready Rust looks like:**
- Type-safe
- Zero-cost abstractions
- Composable pipelines
- Automatic observability
- Beautiful CLI
- Real-world useful
