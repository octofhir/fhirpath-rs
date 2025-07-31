use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use warp::Filter;

#[tokio::main]
async fn main() {
    let criterion_path = get_criterion_path();

    println!("🚀 Starting FHIRPath Benchmark Server");
    println!("📊 Serving reports from: {}", criterion_path.display());

    // Routes
    let routes = home_route()
        .or(benchmark_list_route(criterion_path.clone()))
        .or(performance_stats_route(criterion_path.clone()))
        .or(static_files_route(criterion_path.clone()))
        .or(benchmark_detail_route(criterion_path))
        .with(warp::cors().allow_any_origin());

    println!("🌐 Server running at http://localhost:3030");
    println!("📈 Visit http://localhost:3030 to view benchmark results");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn get_criterion_path() -> PathBuf {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    current_dir.join("target").join("criterion")
}

// Home page route
fn home_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().map(|| warp::reply::html(generate_home_page()))
}

// Benchmark list API route
fn benchmark_list_route(
    criterion_path: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("benchmarks"))
        .and(warp::path::end())
        .map(move || {
            let benchmarks = get_benchmark_list(&criterion_path);
            warp::reply::json(&benchmarks)
        })
}

// Performance stats API route
fn performance_stats_route(
    criterion_path: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("stats"))
        .and(warp::path::end())
        .map(move || {
            let stats = get_performance_stats(&criterion_path);
            warp::reply::json(&stats)
        })
}

// Static files route for serving HTML reports, SVGs, etc.
fn static_files_route(
    criterion_path: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("reports").and(warp::fs::dir(criterion_path))
}

// Benchmark detail route
fn benchmark_detail_route(
    criterion_path: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("benchmark")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .map(move |benchmark_name: String| {
            let benchmark_data = get_benchmark_data(&criterion_path, &benchmark_name);
            warp::reply::html(generate_benchmark_page(&benchmark_name, &benchmark_data))
        })
}

fn get_benchmark_list(criterion_path: &Path) -> Vec<Value> {
    let mut benchmarks = Vec::new();

    if let Ok(entries) = fs::read_dir(criterion_path) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "report" && has_benchmark_data(&entry.path()) {
                    let data = get_benchmark_summary(&entry.path());
                    benchmarks.push(serde_json::json!({
                        "name": name,
                        "path": format!("/reports/{}/report/", name),
                        "summary": data
                    }));
                }
            }
        }
    }

    benchmarks.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    benchmarks
}

fn get_performance_stats(criterion_path: &Path) -> Value {
    let mut stats = serde_json::json!({
        "tokenizer": {
            "ops_per_second": "N/A",
            "avg_time": "N/A",
            "status": "No Data"
        },
        "parser": {
            "ops_per_second": "N/A", 
            "avg_time": "N/A",
            "status": "No Data"
        },
        "evaluator": {
            "ops_per_second": "N/A",
            "avg_time": "N/A", 
            "status": "No Data"
        }
    });

    // Look for benchmark results
    let tokenizer_simple = criterion_path.join("tokenizer").join("tokenize").join("simple").join("new").join("estimates.json");
    let parser_simple = criterion_path.join("parser").join("parse").join("simple").join("new").join("estimates.json");
    let evaluator_simple = criterion_path.join("evaluator").join("evaluate").join("simple").join("new").join("estimates.json");

    // Get tokenizer performance
    if let Ok(tokenizer_data) = fs::read_to_string(&tokenizer_simple) {
        if let Ok(estimates) = serde_json::from_str::<Value>(&tokenizer_data) {
            if let Some(mean_ns) = estimates["mean"]["point_estimate"].as_f64() {
                let ops_per_sec = 1_000_000_000.0 / mean_ns;
                
                let ops_formatted = if ops_per_sec >= 1_000_000.0 {
                    format!("{:.1}M", ops_per_sec / 1_000_000.0)
                } else if ops_per_sec >= 1_000.0 {
                    format!("{:.1}K", ops_per_sec / 1_000.0)
                } else {
                    format!("{:.0}", ops_per_sec)
                };
                
                stats["tokenizer"]["ops_per_second"] = serde_json::json!(ops_formatted);
                stats["tokenizer"]["avg_time"] = serde_json::json!(format!("{:.0}ns", mean_ns));
                stats["tokenizer"]["status"] = serde_json::json!("Fast");
            }
        }
    }

    // Get parser performance  
    if let Ok(parser_data) = fs::read_to_string(&parser_simple) {
        if let Ok(estimates) = serde_json::from_str::<Value>(&parser_data) {
            if let Some(mean_ns) = estimates["mean"]["point_estimate"].as_f64() {
                let ops_per_sec = 1_000_000_000.0 / mean_ns;
                
                let ops_formatted = if ops_per_sec >= 1_000_000.0 {
                    format!("{:.1}M", ops_per_sec / 1_000_000.0)
                } else if ops_per_sec >= 1_000.0 {
                    format!("{:.1}K", ops_per_sec / 1_000.0)
                } else {
                    format!("{:.0}", ops_per_sec)
                };
                
                stats["parser"]["ops_per_second"] = serde_json::json!(ops_formatted);
                stats["parser"]["avg_time"] = serde_json::json!(format!("{:.0}ns", mean_ns));
                stats["parser"]["status"] = serde_json::json!("Fast");
            }
        }
    }

    // Get evaluator performance
    if let Ok(evaluator_data) = fs::read_to_string(&evaluator_simple) {
        if let Ok(estimates) = serde_json::from_str::<Value>(&evaluator_data) {
            if let Some(mean_ns) = estimates["mean"]["point_estimate"].as_f64() {
                let ops_per_sec = 1_000_000_000.0 / mean_ns;
                
                // Format ops/sec based on magnitude for better readability
                let ops_formatted = if ops_per_sec >= 1_000_000.0 {
                    format!("{:.1}M", ops_per_sec / 1_000_000.0)
                } else if ops_per_sec >= 1_000.0 {
                    format!("{:.1}K", ops_per_sec / 1_000.0)
                } else {
                    format!("{:.0}", ops_per_sec)
                };
                
                // Format time based on magnitude
                let time_formatted = if mean_ns >= 1_000_000.0 {
                    format!("{:.0}μs", mean_ns / 1_000.0)
                } else if mean_ns >= 1_000.0 {
                    format!("{:.1}μs", mean_ns / 1_000.0)
                } else {
                    format!("{:.0}ns", mean_ns)
                };
                
                stats["evaluator"]["ops_per_second"] = serde_json::json!(ops_formatted);
                stats["evaluator"]["avg_time"] = serde_json::json!(time_formatted);
                stats["evaluator"]["status"] = serde_json::json!("Active");
            }
        }
    }

    stats
}

fn has_benchmark_data(path: &Path) -> bool {
    path.join("report").join("index.html").exists()
}

fn get_benchmark_summary(path: &Path) -> Value {
    // Try to read benchmark data from estimates.json
    if let Ok(estimates_data) = fs::read_to_string(path.join("new").join("estimates.json")) {
        if let Ok(estimates) = serde_json::from_str::<Value>(&estimates_data) {
            return serde_json::json!({
                "mean": estimates["mean"],
                "median": estimates["median"],
                "std_dev": estimates["std_dev"]
            });
        }
    }

    serde_json::json!({
        "status": "available"
    })
}

fn get_benchmark_data(criterion_path: &Path, benchmark_name: &str) -> Value {
    let benchmark_path = criterion_path.join(benchmark_name);
    get_benchmark_summary(&benchmark_path)
}

fn generate_home_page() -> String {
    r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FHIRPath Parser Benchmarks</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            border-radius: 10px;
            margin-bottom: 30px;
            text-align: center;
        }
        .header h1 {
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }
        .header p {
            margin: 10px 0 0 0;
            opacity: 0.9;
            font-size: 1.1em;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .stat-card {
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
        }
        .stat-header {
            font-size: 1.1em;
            font-weight: 600;
            color: #333;
            margin-bottom: 15px;
        }
        .stat-number {
            font-size: 2.2em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 8px;
        }
        .stat-label {
            color: #666;
            font-size: 0.9em;
            margin-bottom: 8px;
        }
        .stat-status {
            color: #28a745;
            font-size: 0.8em;
            text-transform: uppercase;
            letter-spacing: 1px;
            font-weight: 600;
        }
        .benchmarks {
            background: white;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }
        .benchmarks h2 {
            background: #f8f9fa;
            margin: 0;
            padding: 20px;
            border-bottom: 1px solid #eee;
            cursor: pointer;
            user-select: none;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .benchmarks h2:hover {
            background: #e9ecef;
        }
        .collapse-icon {
            transition: transform 0.3s ease;
        }
        .collapse-icon.collapsed {
            transform: rotate(-90deg);
        }
        .benchmark-content {
            transition: max-height 0.3s ease;
            overflow: hidden;
        }
        .benchmark-content.collapsed {
            max-height: 0;
        }
        .benchmark-list {
            padding: 0;
            margin: 0;
            list-style: none;
        }
        .benchmark-item {
            padding: 20px;
            border-bottom: 1px solid #eee;
            display: flex;
            justify-content: space-between;
            align-items: center;
            transition: background 0.2s;
        }
        .benchmark-item:hover {
            background: #f8f9fa;
        }
        .benchmark-item:last-child {
            border-bottom: none;
        }
        .benchmark-name {
            font-weight: 600;
            color: #333;
        }
        .benchmark-link {
            color: #667eea;
            text-decoration: none;
            padding: 8px 16px;
            border: 1px solid #667eea;
            border-radius: 5px;
            transition: all 0.2s;
        }
        .benchmark-link:hover {
            background: #667eea;
            color: white;
        }
        .loading {
            text-align: center;
            padding: 40px;
            color: #666;
        }
        .footer {
            text-align: center;
            margin-top: 40px;
            padding: 20px;
            color: #666;
            font-size: 0.9em;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 FHIRPath Parser Benchmarks</h1>
        <p>High-Performance Zero-Copy Parser Performance Metrics</p>
    </div>

    <div class="stats">
        <div class="stat-card">
            <div class="stat-header">⚡ Tokenizer</div>
            <div class="stat-number" id="tokenizer-ops">Loading...</div>
            <div class="stat-label" id="tokenizer-time">Loading...</div>
            <div class="stat-status" id="tokenizer-status">Loading...</div>
        </div>
        <div class="stat-card">
            <div class="stat-header">🔍 Parser</div>
            <div class="stat-number" id="parser-ops">Loading...</div>
            <div class="stat-label" id="parser-time">Loading...</div>
            <div class="stat-status" id="parser-status">Loading...</div>
        </div>
        <div class="stat-card">
            <div class="stat-header">⚙️ Evaluator</div>
            <div class="stat-number" id="evaluator-ops">Loading...</div>
            <div class="stat-label" id="evaluator-time">Loading...</div>
            <div class="stat-status" id="evaluator-status">Loading...</div>
        </div>
    </div>

    <div class="benchmarks">
        <h2 onclick="toggleBenchmarks()">
            📊 Available Benchmark Reports
            <span class="collapse-icon collapsed">▼</span>
        </h2>
        <div id="benchmark-content" class="benchmark-content collapsed">
            <div id="benchmark-list" class="loading">Loading benchmarks...</div>
        </div>
    </div>

    <div class="footer">
        <p>Generated by Criterion.rs • FHIRPath Parser Optimization Project</p>
    </div>

    <script>
        async function loadPerformanceStats() {
            try {
                const response = await fetch('/api/stats');
                const stats = await response.json();
                
                // Update tokenizer stats
                document.getElementById('tokenizer-ops').textContent = stats.tokenizer.ops_per_second;
                document.getElementById('tokenizer-time').textContent = stats.tokenizer.avg_time;
                document.getElementById('tokenizer-status').textContent = stats.tokenizer.status;
                
                // Update parser stats
                document.getElementById('parser-ops').textContent = stats.parser.ops_per_second;
                document.getElementById('parser-time').textContent = stats.parser.avg_time;
                document.getElementById('parser-status').textContent = stats.parser.status;
                
                // Update evaluator stats
                document.getElementById('evaluator-ops').textContent = stats.evaluator.ops_per_second;
                document.getElementById('evaluator-time').textContent = stats.evaluator.avg_time;
                document.getElementById('evaluator-status').textContent = stats.evaluator.status;
                
            } catch (error) {
                console.error('Failed to load performance stats:', error);
                // Set error state for all components
                ['tokenizer', 'parser', 'evaluator'].forEach(component => {
                    document.getElementById(component + '-ops').textContent = 'N/A';
                    document.getElementById(component + '-time').textContent = 'N/A';
                    document.getElementById(component + '-status').textContent = 'Error';
                });
            }
        }

        function toggleBenchmarks() {
            const content = document.getElementById('benchmark-content');
            const icon = document.querySelector('.collapse-icon');
            
            content.classList.toggle('collapsed');
            icon.classList.toggle('collapsed');
        }

        async function loadBenchmarks() {
            try {
                const response = await fetch('/api/benchmarks');
                const benchmarks = await response.json();
                
                const listElement = document.getElementById('benchmark-list');
                
                if (benchmarks.length === 0) {
                    listElement.innerHTML = '<div class="loading">No benchmarks found. Run `cargo bench` to generate reports.</div>';
                    return;
                }
                
                const listHTML = benchmarks.map(benchmark => `
                    <div class="benchmark-item">
                        <div class="benchmark-name">${benchmark.name.replace(/_/g, ' ').replace(/\\b\\w/g, l => l.toUpperCase())}</div>
                        <a href="${benchmark.path}" class="benchmark-link" target="_blank">View Report</a>
                    </div>
                `).join('');
                
                listElement.innerHTML = `<div class="benchmark-list">${listHTML}</div>`;
                
            } catch (error) {
                console.error('Failed to load benchmarks:', error);
                document.getElementById('benchmark-list').innerHTML = '<div class="loading">Failed to load benchmarks.</div>';
            }
        }
        
        // Load both performance stats and benchmarks
        Promise.all([loadPerformanceStats(), loadBenchmarks()]);
    </script>
</body>
</html>
"#.to_string()
}

fn generate_benchmark_page(name: &str, data: &Value) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Benchmark: {}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            background: #667eea;
            color: white;
            padding: 30px;
            border-radius: 10px;
            margin-bottom: 30px;
        }}
        .data {{
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        pre {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>📊 {}</h1>
        <p>Benchmark Details</p>
    </div>
    
    <div class="data">
        <h2>Performance Data</h2>
        <pre>{}</pre>
        
        <p><a href="/">← Back to Benchmark List</a></p>
    </div>
</body>
</html>
"#,
        name,
        name,
        serde_json::to_string_pretty(data).unwrap_or_else(|_| "No data available".to_string())
    )
}
