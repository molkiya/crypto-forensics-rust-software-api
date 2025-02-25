mod infrastructure;
mod application;

use actix_web::{Responder, web, get, post, HttpRequest, HttpResponse, HttpServer, App};
use tokio::fs::{read_to_string, create_dir_all};
use serde_json::{json, Map, Number, Value};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::env;
use std::net::{TcpListener};
use urlencoding;
use open;
use infrastructure::constants::{UNIX_START_PORT,
                                UNIX_END_PORT,
                                WINDOWS_START_PORT,
                                WINDOWS_END_PORT};
use csv::Reader;
use tera::{Tera, Context};

const DATA_DIR: &str = "./src/data";

fn redirect_to_error_page(error_message: &str) -> HttpResponse {
    println!("{}", error_message);
    HttpResponse::Found()
        .header("Location", format!("/error.html?error={}", urlencoding::encode(error_message)))
        .finish()
}

#[get("/")]
async fn index(_req: HttpRequest) -> HttpResponse {
    let current_dir = env::current_dir().unwrap();
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

fn get_data_path(folder_name: &str, file_name: &str) -> PathBuf {
    let mut path = env::current_dir().expect("Не удалось получить текущую директорию");
    path.push("src/data");
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
struct Node {
    id: String,
}

#[derive(Debug, Serialize)]
struct Edge {
    from: String,
    to: String,
    txId: String
}

fn plot_graph(folder_name: &str) -> Result<Value, std::io::Error> {
    let mut nodes_set = HashSet::new();
    let mut edges = Vec::new();
    let mut tx_map: HashMap<String, String> = HashMap::new();

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
            edges.push(Edge { from: from.clone(), to: record.output_address, txId: record.txId });
        }
    }

    // Формируем список узлов
    let nodes: Vec<Node> = nodes_set.into_iter().map(|id| Node { id }).collect();

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

    // Загружаем `graph_template.html`
    let graph_template_path = Path::new("static/graph_template.html");
    let graph_template_content = match read_to_string(graph_template_path).await {
        Ok(content) => content,
        Err(err) => return redirect_to_error_page(&format!("Ошибка загрузки HTML-шаблона: {:?}", err)),
    };

    let folder_path = Path::new(DATA_DIR).join(folder_name);
    if let Err(err) = create_dir_all(&folder_path).await {
        return redirect_to_error_page(&format!("Не удалось создать папку: {}", err));
    }

    // Создаем контекст для `graph_template`
    // Генерируем данные графа
    let graph_data = match plot_graph(folder_name) {
        Ok(data) => data,
        Err(err) => return redirect_to_error_page(&format!("Ошибка генерации графа: {:?}", err)),
    };

    // Создаем контекст для шаблона
    let mut graph_context = Context::new();

    if let Some(nodes) = graph_data.get("nodes") {
        let nodes_json = serde_json::to_string(nodes).unwrap_or_else(|_| "[]".to_string());
        println!("{}", &nodes_json);
        graph_context.insert("nodes", &nodes_json);
    }
    if let Some(edges) = graph_data.get("edges") {
        let edges_json = serde_json::to_string(edges).unwrap_or_else(|_| "[]".to_string());
        graph_context.insert("edges", &edges_json);
    }

    // Рендерим `graph_template`
    let graph_rendered = match Tera::one_off(&graph_template_content, &graph_context, true) {
        Ok(html) => html,
        Err(err) => return redirect_to_error_page(&format!("Ошибка рендеринга шаблона графа: {:?}", err)),
    };

    // Загружаем `main_template.html`
    let main_template_path = Path::new("static/analysis.html");
    let main_template_content = match read_to_string(main_template_path).await {
        Ok(content) => content,
        Err(err) => return redirect_to_error_page(&format!("Ошибка загрузки основного HTML-шаблона: {:?}", err)),
    };

    // Подставляем `graph_rendered` в `PLOT_DATA`
    let final_html = main_template_content.replace("{PLOT_DATA}", &graph_rendered);

    // Возвращаем HTML
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(final_html)
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
    let mut available_port: Option<u16> = None; // Declare as Option<u16>

    let os = env::consts::OS;
    println!("Running on OS: {}", os);

    // Detect the OS and find an available port accordingly
    match os {
        "windows" => {
            available_port = find_available_port(WINDOWS_START_PORT, WINDOWS_END_PORT).await;
            println!("Windows OS detected");
        }
        "linux" | "macos" => {
            available_port = find_available_port(UNIX_START_PORT, UNIX_END_PORT).await;
            println!("Unix-based OS detected");
        }
        _ => {
            println!("Unknown OS detected");
        }
    }

    match available_port {
        Some(port) => {
            println!("Found available port: {}", port);
            let server = HttpServer::new(|| App::new().service(index).service(confirm_file))
                .bind(("127.0.0.1", port))?
                .workers(1)
                .run();

            open::that(format!("http://127.0.0.1:{}/", port)).expect("Failed to open browser");
            server.await
        }
        None => {
            println!("No available ports found in the specified range");
            Ok(())
        }
    }
}
