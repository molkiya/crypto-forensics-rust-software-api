use actix_web::{Responder, web, get, post, HttpRequest, HttpResponse, HttpServer, App};
use tokio::fs::{read_to_string, create_dir_all};
use std::collections::HashMap;
use std::path::Path;
use std::env;
use std::net::{TcpListener};
use plotly::{Scatter, Plot};
use urlencoding;
use open;

const DATA_DIR: &str = "./src/data";

const UNIX_START_PORT: u16 = 8000;
const UNIX_END_PORT: u16 = 8999;

const WINDOWS_START_PORT: u16 = 49152;
const WINDOWS_END_PORT: u16 = 65535;

fn redirect_to_error_page(error_message: &str) -> HttpResponse {
    HttpResponse::Found()
        .header("Location", format!("/error.html?error={}", urlencoding::encode(error_message)))
        .finish()
}

async fn plot_generation() -> Result<String, std::io::Error> {
    // Генерируем график
    let trace = Scatter::new(vec![1, 2, 3, 4, 5], vec![10, 15, 13, 17, 21]);
    let mut plot = Plot::new();
    plot.add_trace(trace);

    // Получаем HTML-график
    let graph_html = plot.to_html();

    // Читаем шаблон
    let template_path = "static/analysis.html";
    let template = tokio::fs::read_to_string(template_path).await?;

    // Вставляем график в шаблон
    let result_html = template.replace("{PLOT_DATA}", &graph_html);

    Ok(result_html)
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

#[post("/confirm")]
async fn confirm_file(form: web::Form<HashMap<String, String>>) -> impl Responder {
    let folder_name = match form.get("inputText") {
        Some(name) => name,
        None => return redirect_to_error_page("Поле inputText отсутствует"),
    };

    if folder_name.len() < 14 || folder_name.len() > 74 {
        return redirect_to_error_page("Название папки должно содержать от 14 до 74 символов");
    }

    let folder_path = Path::new(DATA_DIR).join(folder_name);
    if let Err(err) = create_dir_all(&folder_path).await {
        return redirect_to_error_page(&format!("Не удалось создать папку: {}", err));
    }

    match plot_generation().await {
        Ok(html_content) => HttpResponse::Ok().content_type("text/html").body(html_content),
        Err(_) => redirect_to_error_page("Ошибка генерации графика"),
    }
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
            // Now you can bind to this port (or start your server with it)
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
