mod application;
mod infrastructure;
mod services;

use services::explorer::explorer_client::get_or_init_client;
use application::services::transaction::transaction_info::get_transaction_info;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализируем клиент explorer
    get_or_init_client().await?;
    
    // Получаем конфигурацию
    let config = Config::from_env();
    
    // Получаем информацию о транзакции
    let features = get_transaction_info(&config.test_tx_id).await?;
    
    println!("Transaction Features:");
    println!("  Inputs: {}", features.n_inputs);
    println!("  Outputs: {}", features.n_outputs);
    println!("  Input Value Sum: {} BTC", features.input_value_sum);
    println!("  Output Value Sum: {} BTC", features.output_value_sum);
    println!("  Transaction Fee: {} BTC", features.transaction_fee);
    println!("  Avg Input Value: {} BTC", features.avg_input_value);
    println!("  Avg Output Value: {} BTC", features.avg_output_value);
    
    Ok(())
}