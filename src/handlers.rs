use std::{io::Result, path::Path};
use tokio::{fs, net::TcpStream};

pub async fn handle_get_request(stream: &mut TcpStream, path: &str, log: &crate::utils::LogEntry) -> Result<()> {
    let status = match serve_static_file(stream, path).await {
        Ok(code) => code,
        Err(_) => {
            crate::utils::send_500_response(stream, b"500 Internal Server Error").await?;
            "500".to_string()
        }
    };
    log.log(&status);
    Ok(())
}

pub async fn handle_post_request(
    stream: &mut TcpStream,
    path: &str,
    body: &str,
    log: &crate::utils::LogEntry,
) -> Result<()> {
    match path {
        "/api/register" => handle_register(stream, body, log).await,
        "/api/login" => handle_login(stream, body, log).await,
        _ => {
            crate::utils::send_400_response(stream, b"400 Bad Request").await?;
            log.log("404");
            Ok(())
        }
    }
}

pub async fn serve_static_file(stream: &mut TcpStream, path: &str) -> Result<String> {
    // 移除开头的斜杠，构建文件路径
    let path = path.trim_start_matches('/');
    let file_path = Path::new("public").join(path);

    // 检查文件是否存在
    if !file_path.exists() {
        crate::utils::send_response(stream, "404 Not Found", b"File not found", "text/plain", None).await?;
        return Ok("404".to_string());
    }

    // 读取文件内容
    let contents = match fs::read(&file_path).await {
        Ok(c) => c,
        Err(_) => {
            crate::utils::send_response(stream, "500 Internal Server Error", b"Error reading file", "text/plain", None).await?;
            return Ok("500".to_string());
        }
    };

    // 确定MIME类型
    let mime_type = match file_path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    // 发送文件内容
    crate::utils::send_response(stream, "200 OK", &contents, mime_type, None).await?;
    Ok("200".to_string())
}

async fn handle_login(stream: &mut TcpStream, body: &str, log: &crate::utils::LogEntry)-> Result<()> {
    Ok(())
}
async fn handle_register(
    stream: &mut TcpStream,
    body: &str,
    log: &crate::utils::LogEntry,
) -> Result<()> {
    Ok(())
}
/*
async fn handle_register(
    stream: &mut TcpStream,
    body: &str,
    log: &crate::utils::LogEntry,
) -> Result<()> {
    // 解析JSON数据
    let data: serde_json::Value = match serde_json::from_str(body) {
        Ok(d) => d,
        Err(_) => {
            send_json_response(stream, 400, serde_json::json!({"error": "Invalid JSON format"}))?;
            log.log("400");
            return Ok(());
        }
    };

    // 提取字段
    let username = data.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let password = data.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // 验证必填字段
    if username.is_empty() || email.is_empty() || password.is_empty() {
        send_json_response(stream, 400, serde_json::json!({"error": "Username, email and password are required"}))?;
        log.log("400");
        return Ok(());
    }

    // 验证邮箱格式（简单验证）
    if !email.contains('@') || !email.contains('.') {
        send_json_response(stream, 400, serde_json::json!({"error": "Invalid email format"}))?;
        log.log("400");
        return Ok(());
    }

    // 验证密码长度
    if password.len() < 6 {
        send_json_response(stream, 400, serde_json::json!({"error": "Password must be at least 6 characters"}))?;
        log.log("400");
        return Ok(());
    }

    // 直接调用异步数据库操作
    match crate::database::register_user(&username, &email, &password).await {
        Ok(true) => {
            send_json_response(stream, 201, serde_json::json!({"message": "User registered successfully"}))?;
            log.log("201");
        },
        Ok(false) => {
            send_json_response(stream, 409, serde_json::json!({"error": "Username or email already exists"}))?;
            log.log("409");
        },
        Err(e) => {
            eprintln!("Database error during registration: {}", e);
            send_json_response(stream, 500, serde_json::json!({"error": "Registration failed"}))?;
            log.log("500");
        }
    }
    
    Ok(())
}

async fn handle_login(
    stream: &mut TcpStream,
    body: &str,
    log: &crate::utils::LogEntry,
) -> Result<()> {
    // 解析JSON数据
    let data: serde_json::Value = match serde_json::from_str(body) {
        Ok(d) => d,
        Err(_) => {
            send_json_response(stream, 400, serde_json::json!({"error": "Invalid JSON format"}))?;
            log.log("400");
            return Ok(());
        }
    };

    // 提取字段
    let username = data.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let password = data.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // 验证必填字段
    if username.is_empty() || password.is_empty() {
        send_json_response(stream, 400, serde_json::json!({"error": "Username and password are required"}))?;
        log.log("400");
        return Ok(());
    }

    // 直接调用异步数据库操作
    match crate::database::login_user(&username, &password).await {
        Ok(Some(user_name)) => {
            send_json_response(stream, 200, serde_json::json!({
                "message": "Login successful", 
                "user": user_name
            }))?;
            log.log("200");
        },
        Ok(None) => {
            send_json_response(stream, 401, serde_json::json!({"error": "Invalid username or password"}))?;
            log.log("401");
        },
        Err(e) => {
            eprintln!("Database error during login: {}", e);
            send_json_response(stream, 500, serde_json::json!({"error": "Login failed"}))?;
            log.log("500");
        }
    }
    
    Ok(())
}
*/