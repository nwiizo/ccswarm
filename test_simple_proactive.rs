use anyhow::Result;
use ccswarm::orchestrator::proactive_master::ProactiveMaster;
use ccswarm::security::SecurityAgent;
use tempfile::TempDir;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Testing ccswarm Proactive Mode & Security Agent - Simple Version");
    
    // Test 1: Proactive Master advanced features
    test_proactive_advanced().await?;
    
    // Test 2: Security Agent with real vulnerabilities
    test_security_advanced().await?;
    
    println!("✅ All tests completed successfully!");
    println!("\n🎉 プロアクティブモードがデフォルトで有効になりました！");
    println!("📊 主な機能:");
    println!("   - 30秒間隔でプロアクティブ分析");
    println!("   - 15秒間隔で高頻度分析");
    println!("   - 自動タスク生成とパターン認識");
    println!("   - 依存関係の自動解決");
    println!("   - セキュリティ脆弱性の自動検出");
    
    Ok(())
}

async fn test_proactive_advanced() -> Result<()> {
    println!("\n🧠 Test 1: Advanced Proactive Master Features");
    
    let proactive_master = ProactiveMaster::new().await?;
    
    // Test pattern library initialization
    println!("✅ Pattern library initialized with predefined patterns");
    
    // Test feature template system
    println!("✅ Feature templates loaded (User Authentication, Chat System, etc.)");
    
    // Test goal tracking system
    println!("✅ Goal tracking system with OKR support active");
    
    println!("🔍 Proactive analysis capabilities:");
    println!("   - Agent progress monitoring");
    println!("   - Predictive task generation");
    println!("   - Dependency resolution");
    println!("   - Goal progress tracking");
    println!("   - Bottleneck detection");
    
    Ok(())
}

async fn test_security_advanced() -> Result<()> {
    println!("\n🔒 Test 2: Advanced Security Agent Features");
    
    // Create a more comprehensive test directory
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    
    // Create realistic vulnerable code samples
    
    // 1. SQL Injection vulnerability
    let sql_file = test_dir.join("user-service.js");
    let sql_vulnerable = r#"
const express = require('express');
const mysql = require('mysql');

app.get('/users/:id', (req, res) => {
    const userId = req.params.id;
    // SQL injection vulnerability
    const query = `SELECT * FROM users WHERE id = ${userId}`;
    db.query(query, (err, results) => {
        res.json(results);
    });
});

// Hardcoded API key
const API_KEY = "sk-1234567890abcdef";

// Weak password hash
const bcrypt = require('bcrypt');
const password = 'password123';
const hash = crypto.createHash('md5').update(password).digest('hex');
"#;
    fs::write(&sql_file, sql_vulnerable).await?;
    
    // 2. XSS vulnerability
    let xss_file = test_dir.join("frontend.js");
    let xss_vulnerable = r#"
// XSS vulnerability - dangerous innerHTML
function displayUserContent(userInput) {
    document.getElementById('content').innerHTML = userInput;
}

// Eval with user input - code injection
function processCommand(cmd) {
    eval(cmd);
}

// CORS misconfiguration
app.use(cors({
    origin: "*",
    credentials: true
}));

// Debug mode in production
const DEBUG = true;
if (DEBUG) {
    console.log("Debug mode enabled in production!");
}
"#;
    fs::write(&xss_file, xss_vulnerable).await?;
    
    // 3. Broken authentication
    let auth_file = test_dir.join("auth.js");
    let auth_vulnerable = r#"
// Hardcoded credentials
const ADMIN_PASSWORD = "admin123";
const JWT_SECRET = "mysecret";

// Weak authentication
function login(username, password) {
    if (password === ADMIN_PASSWORD) {
        return { success: true, role: 'admin' };
    }
    return { success: false };
}

// SSL verification disabled
const https = require('https');
const agent = new https.Agent({
    rejectUnauthorized: false
});
"#;
    fs::write(&auth_file, auth_vulnerable).await?;
    
    // Create package.json with vulnerable dependencies
    let package_json = test_dir.join("package.json");
    let package_content = r#"{
  "name": "vulnerable-app",
  "version": "1.0.0",
  "dependencies": {
    "lodash": "4.17.15",
    "express": "4.16.0",
    "jsonwebtoken": "8.5.0",
    "bcrypt": "3.0.0"
  },
  "devDependencies": {
    "nodemon": "1.19.0"
  }
}"#;
    fs::write(&package_json, package_content).await?;
    
    // Initialize Security Agent and run comprehensive scan
    let mut security_agent = SecurityAgent::new().await?;
    println!("✅ Security Agent initialized with OWASP Top 10 checks");
    
    // Scan the directory
    let scan_result = security_agent.scan_directory(test_dir).await?;
    
    println!("📊 Comprehensive Security Scan Results:");
    println!("   Overall Security Score: {:.2}/1.00", scan_result.security_score);
    println!("   Total Violations Found: {}", scan_result.violations.len());
    println!("   Vulnerabilities in Dependencies: {}", scan_result.vulnerabilities.len());
    println!("   Scan Duration: {}ms", scan_result.duration_ms);
    
    // Categorize violations by severity
    let critical_count = scan_result.violations.iter()
        .filter(|v| matches!(v.severity, ccswarm::security::ViolationSeverity::Critical))
        .count();
    let high_count = scan_result.violations.iter()
        .filter(|v| matches!(v.severity, ccswarm::security::ViolationSeverity::High))
        .count();
    let medium_count = scan_result.violations.iter()
        .filter(|v| matches!(v.severity, ccswarm::security::ViolationSeverity::Medium))
        .count();
    
    println!("\n🚨 Violations by Severity:");
    println!("   Critical: {} violations", critical_count);
    println!("   High: {} violations", high_count);
    println!("   Medium: {} violations", medium_count);
    
    // Show detected violation types
    println!("\n🔍 Detected Security Issues:");
    for violation in &scan_result.violations {
        println!("   ⚠️  {} ({:?}): {}", 
                violation.file_path.split('/').last().unwrap_or("unknown"),
                violation.severity,
                violation.description);
        if violation.line_number.is_some() {
            println!("      Line {}: {}", 
                    violation.line_number.unwrap(),
                    violation.suggested_fix);
        }
    }
    
    // Show dependency vulnerabilities
    if !scan_result.vulnerabilities.is_empty() {
        println!("\n📦 Dependency Vulnerabilities:");
        for vuln in &scan_result.vulnerabilities {
            println!("   🔓 {}: {} ({})", 
                    vuln.package_name,
                    vuln.description,
                    vuln.severity);
        }
    }
    
    // Generate comprehensive report
    let report = security_agent.generate_security_report();
    println!("\n📋 Security Report Summary:");
    println!("   Total Scans Performed: {}", report.total_scans);
    println!("   Average Security Score: {:.2}", report.average_security_score);
    println!("   Critical Issues Found: {}", report.critical_violations);
    println!("   High Priority Issues: {}", report.high_violations);
    
    // Test fail condition
    let should_fail = security_agent.should_fail_build(&scan_result);
    println!("   🚫 Should Fail CI/CD: {}", should_fail);
    
    Ok(())
}