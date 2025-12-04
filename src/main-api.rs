mod infrastructure;
mod application;
mod services;

use actix_web::{Responder, web, get, post, HttpRequest, HttpResponse, HttpServer, App};
use tokio::fs::{read_to_string, metadata, DirEntry};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::env;
use env_logger;
use std::net::{TcpListener};
use urlencoding;
use open;
use csv::Reader;
use tera::{Tera, Context};
use crate::infrastructure::constants::{END_PORT, START_PORT};
use std::time::{UNIX_EPOCH};

use crate::infrastructure::config::Config;

/// Перенаправляет на страницу ошибки с сообщением
fn redirect_to_error_page(error_message: &str) -> HttpResponse {
    eprintln!("Error: {}", error_message);
    HttpResponse::Found()
        .append_header(("Location", format!("/error.html?error={}", urlencoding::encode(error_message))))
        .finish()
}

/// Ищет запись в CSV файле по ключу
///
/// # Arguments
/// * `file_name` - Имя CSV файла
/// * `key` - Ключ для поиска (первая колонка)
/// * `data_folder` - Папка с данными
///
/// # Returns
/// `Option<HashMap<String, String>>` - Найденные данные или None
fn find_in_csv(file_name: &str, key: &str, data_folder: &str) -> Option<HashMap<String, String>> {
    let path = get_data_path(data_folder, file_name);
    if !path.exists() {
        return None;
    }
    let mut rdr = Reader::from_path(&path).ok()?;
    let headers = rdr.headers().ok()?.clone();
    for result in rdr.records() {
        let record = result.ok()?;
        if record.get(0)? == key {
            let mut data = HashMap::new();
            for (i, field) in headers.iter().enumerate() {
                data.insert(field.to_string(), record.get(i)?.to_string());
            }
            return Some(data);
        }
    }
    None
}

struct AppState {
    tera: Tera,
}

/// Получает информацию о транзакции по её ID
///
/// # Arguments
/// * `path` - Path параметр с ID транзакции
///
/// # Returns
/// HTML страницу с информацией о транзакции или страницу ошибки
#[get("/tx/{tx_id}")]
async fn get_transaction(
    path: web::Path<String>
) -> impl Responder {
    let tx_id = path.into_inner();
    let config = Config::from_env();
    let file_path = get_data_path(&config.default_data_folder, "elliptic_txs_features.csv");

    let file_path_str = match file_path.to_str() {
        Some(s) => s,
        None => return redirect_to_error_page("Invalid file path encoding"),
    };

    match find_in_csv(file_path_str, &tx_id, &config.default_data_folder) {
        Some(data) => {
            let mut ctx = Context::new();
            ctx.insert("data", &data);

            let analysis_template_path = Path::new("static/tx.html");
            let analysis_template_content = match read_to_string(analysis_template_path).await {
                Ok(content) => content,
                Err(err) => return redirect_to_error_page(&format!("Ошибка загрузки шаблона анализа: {:?}", err)),
            };

            match Tera::one_off(&analysis_template_content, &ctx, true) {
                Ok(html) => HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html),
                Err(err) => redirect_to_error_page(&format!("Ошибка рендеринга шаблона: {:?}", err)),
            }
        }
        None => HttpResponse::NotFound().content_type("text/html; charset=utf-8").body("Транзакция не найдена"),
    }
}


/// Получает информацию об адресе по его значению
///
/// # Arguments
/// * `path` - Path параметр с Bitcoin адресом
///
/// # Returns
/// HTML страницу с информацией об адресе или страницу ошибки
#[get("/address/{address}")]
async fn get_address(
    path: web::Path<String>
) -> impl Responder {
    let address = path.into_inner();
    let config = Config::from_env();
    let file_path = get_data_path(&config.default_data_folder, "wallets_features_classes_combined.csv");

    let file_path_str = match file_path.to_str() {
        Some(s) => s,
        None => return redirect_to_error_page("Invalid file path encoding"),
    };

    match find_in_csv(file_path_str, &address, &config.default_data_folder) {
        Some(data) => {
            let mut ctx = Context::new();
            ctx.insert("data", &data);

            let analysis_template_path = Path::new("static/address.html");
            let analysis_template_content = match read_to_string(analysis_template_path).await {
                Ok(content) => content,
                Err(err) => return redirect_to_error_page(&format!("Ошибка загрузки шаблона анализа: {:?}", err)),
            };

            match Tera::one_off(&analysis_template_content, &ctx, true) {
                Ok(html) => HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html),
                Err(err) => redirect_to_error_page(&format!("Ошибка рендеринга шаблона: {:?}", err)),
            }
        }
        None => HttpResponse::NotFound().content_type("text/html; charset=utf-8").body("Адрес не найден"),
    }
}

async fn process_entry(entry: DirEntry) {
    let path = entry.path();
    if path.is_dir() {
        if let Ok(metadata) = metadata(&path).await {
            if let Ok(created_time) = metadata.created() {
                let since_epoch = created_time.duration_since(UNIX_EPOCH).unwrap_or_default();
                println!(
                    "Папка: {:?}, Дата создания: {} секунд с 1970-01-01",
                    path.file_name().unwrap_or_default(),
                    since_epoch.as_secs()
                );
            }
        }
    }
}

/// Главная страница приложения
///
/// # Returns
/// HTML главной страницы или страницу ошибки
#[get("/")]
async fn index(_req: HttpRequest) -> HttpResponse {
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to get current directory: {}", e);
            return redirect_to_error_page("Failed to get current directory");
        }
    };
    let html_path = current_dir.join("static/main.html");

    match read_to_string(html_path).await {
        Ok(html_content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html_content),
        Err(_) => redirect_to_error_page("Ошибка загрузки главной страницы"),
    }
}

#[derive(Debug)]
struct Transaction {
    tx_id: String,
    class: String,
    features: Vec<f64>,
}

/// Формирует путь к файлу данных
///
/// # Arguments
/// * `folder_name` - Имя папки с данными
/// * `file_name` - Имя файла
///
/// # Returns
/// `PathBuf` - Полный путь к файлу
fn get_data_path(folder_name: &str, file_name: &str) -> PathBuf {
    let config = Config::from_env();
    let mut path = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    path.push(&config.data_dir);
    path.push(folder_name);
    path.push(file_name);
    path
}

#[derive(Debug, Deserialize)]
struct AddrTx {
    input_address: String,
    txId: String,
}

#[derive(Debug, Deserialize)]
struct TxAddr {
    txId: String,
    output_address: String,
}

#[derive(Debug, Serialize)]
struct NormalNode {
    fill: String
}

#[derive(Debug, Serialize)]
struct Node {
    id: String,
    normal: NormalNode
}

#[derive(Debug, Serialize)]
struct Edge {
    from: String,
    to: String,
    id: String,
    normal: NormalEdge
}

#[derive(Debug, Serialize)]
struct NormalEdge {
    stroke: StrokeEdge,
}

#[derive(Debug, Serialize)]
struct StrokeEdge {
    color: String
}

fn plot_graph(folder_name: &str) -> Result<Value, std::io::Error> {
    let mut nodes_set = HashSet::new();
    let mut edges = Vec::new();
    let mut tx_map: HashMap<String, String> = HashMap::new();
    let mut tx_classes: HashMap<String, String> = HashMap::new(); // Хранение классов транзакций

    // Читаем elliptic_txs_classes.csv
    let path = get_data_path(folder_name, "elliptic_txs_classes.csv");
    if path.exists() {
        let mut rdr = Reader::from_path(&path)?;
        for result in rdr.records() {
            let record = result?;
            let tx_id = record.get(0).unwrap_or("").to_string();
            let class = record.get(1).unwrap_or("").to_string();
            tx_classes.insert(tx_id, class);
        }
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File {:?} not found", path)));
    }

    let mut node_classes: HashMap<String, String> = HashMap::new(); // Хранение классов узлов

    // Читаем wallet_features_classes_combined.csv
    let path = get_data_path(folder_name, "wallets_features_classes_combined.csv");
    if path.exists() {
        let mut rdr = Reader::from_path(&path)?;
        for result in rdr.records() {
            let record = result?;
            let address = record.get(0).unwrap_or("").to_string();
            let class = record.get(2).unwrap_or("").to_string(); // Третий столбец
            node_classes.insert(address, class);
        }
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File {:?} not found", path)));
    }

    // Читаем AddrTx_edgelist.csv
    let path = get_data_path(folder_name,"AddrTx_edgelist.csv");
    if !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File {:?} not found", path)));
    }
    let mut rdr = Reader::from_path(&path)?;
    let mut records = rdr.deserialize::<AddrTx>();
    while let Some(record) = records.next() {
        let record = record?;
        nodes_set.insert(record.input_address.clone());
        tx_map.insert(record.txId.clone(), record.input_address);
    }

    // Читаем TxAddr_edgelist.csv
    let path = get_data_path(folder_name,"TxAddr_edgelist.csv");
    if !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File {:?} not found", path)));
    }
    let mut rdr = Reader::from_path(&path)?;
    let mut records = rdr.deserialize::<TxAddr>();
    while let Some(record) = records.next() {
        let record = record?;
        nodes_set.insert(record.output_address.clone());
        if let Some(from) = tx_map.get(&record.txId) {
            let tx_class = tx_classes.get(&record.txId).map(|s| s.as_str()).unwrap_or("unknown");
            let fill_color = if tx_class == "unknown" {
                "#00FF00" // Зеленый
            } else if tx_class == "2" {
                "#FF0000" // Красный
            } else {
                "#CCCCCC" // Серый по умолчанию
            };

            edges.push(Edge {
                from: from.clone(),
                to: record.output_address,
                id: record.txId,
                normal: NormalEdge { stroke: StrokeEdge { color: String::from(fill_color) }},
            });
        }
    }

    // Формируем список узлов
    let nodes: Vec<Node> = nodes_set.into_iter().map(|id| {
        let node_class = node_classes.get(&id).map(|s| s.as_str()).unwrap_or("unknown");
        let fill_color = if node_class == "3" {
            "#00FF00" // Зеленый
        } else if node_class == "2" {
            "#CCCCCC" // Серый
        } else {
            "#FFFFFF" // Обычный белый
        };

        Node { id, normal: NormalNode { fill: String::from(fill_color) } }
    }).collect();

    // Создаём объект графа
    Ok(json!({ "nodes": nodes, "edges": edges }))
}

#[post("/confirm")]
async fn confirm_file(form: web::Form<HashMap<String, String>>) -> impl Responder {
    // Получаем название папки
    let folder_name = match form.get("inputText") {
        Some(name) => name,
        None => return redirect_to_error_page("Поле inputText отсутствует"),
    };

    // Проверка длины
    if folder_name.len() < 14 || folder_name.len() > 74 {
        return redirect_to_error_page("Название папки должно содержать от 14 до 74 символов");
    }

    // Загружаем template для анализа
    let analysis_template_path = Path::new("static/analysis.html");
    let analysis_template_content = match read_to_string(analysis_template_path).await {
        Ok(content) => content,
        Err(err) => return redirect_to_error_page(&format!("Ошибка загрузки шаблона анализа: {:?}", err)),
    };

    // Генерация данных графа (AnyChart)
    let graph_data = match plot_graph(folder_name) {
        Ok(data) => data,
        Err(err) => return redirect_to_error_page(&format!("Ошибка генерации графа: {:?}", err)),
    };

    // Создаем контекст для шаблона
    let mut graph_context = Context::new();
    if let Some(nodes) = graph_data.get("nodes") {
        let nodes_json = serde_json::to_string(nodes).unwrap_or_else(|_| "[]".to_string());
        graph_context.insert("nodes", &nodes_json);
    }
    if let Some(edges) = graph_data.get("edges") {
        let edges_json = serde_json::to_string(edges).unwrap_or_else(|_| "[]".to_string());
        graph_context.insert("edges", &edges_json);
    }

    // Рендерим граф в шаблоне
    let graph_rendered = match Tera::one_off(&analysis_template_content, &graph_context, true) {
        Ok(html) => html,
        Err(err) => return redirect_to_error_page(&format!("Ошибка рендеринга графа в шаблоне: {:?}", err)),
    };

    // Возвращаем итоговый HTML
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(graph_rendered)
}


async fn find_available_port(start_port: u16, end_port: u16) -> Option<u16> {
    for port in start_port..=end_port {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr);

        match listener {
            Ok(_) => {
                // Port is available
                return Some(port);
            }
            Err(_) => {
                // Port is already in use
                continue;
            }
        }
    }

    None
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let available_port: Option<u16> = find_available_port(START_PORT, END_PORT).await;

    match available_port {
        Some(port) => {
            println!("Found available port: {}", port);
            let server = HttpServer::new(|| App::new()
                .service(index)
                .service(confirm_file)
                .service(get_transaction)
                .service(get_address)
            )
                .bind(("127.0.0.1", port))?
                .workers(1)
                .run();

            if let Err(e) = open::that(format!("http://127.0.0.1:{}/", port)) {
                eprintln!("Warning: Failed to open browser: {}", e);
            }
            server.await
        }
        None => {
            println!("No available ports found in the specified range");
            Ok(())
        }
    }
}
