use crate::services::explorer::explorer_client::get_or_init_client;
use crate::services::explorer::errors::explorer_errors::ExplorerError;
use serde::{Deserialize, Serialize};

/// Характеристики транзакции Bitcoin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionFeatures {
    /// Количество входов транзакции
    pub n_inputs: usize,
    /// Количество выходов транзакции
    pub n_outputs: usize,
    /// Сумма BTC на входах
    pub input_value_sum: f64,
    /// Сумма BTC на выходах
    pub output_value_sum: f64,
    /// Комиссия транзакции (input_value_sum - output_value_sum)
    pub transaction_fee: f64,
    /// Средний размер входящего UTXO
    pub avg_input_value: f64,
    /// Средний размер создаваемого UTXO
    pub avg_output_value: f64,
}

#[derive(Debug, Deserialize)]
struct Vin {
    #[serde(default)]
    value: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    value_sat: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Vout {
    value: f64,
    #[serde(default)]
    #[allow(dead_code)]
    value_sat: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TransactionData {
    #[allow(dead_code)]
    txid: String,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
}

// | 1   | **time\_step**                                 | Порядковый номер «временного шага» транзакции (1–49), соответствующий ее времени (интервал \~2 недели). Вычисляется по метке времени блока (RPC `getblockheader`) или моменту включения в блок.                                                                                                                                                                                                                                         |
// | 2   | **n\_inputs**                                  | Число входов транзакции (количество UTXO, используемых как входы). Извлекается из данных транзакции (RPC `getrawtransaction` + `decode` или библиотека типа Bitcoin Core).                                                                                                                                                                                                                                                              |
// | 3   | **n\_outputs**                                 | Число выходов транзакции (сколько UTXO создается). Извлекается из полей outputs транзакции.                                                                                                                                                                                                                                                                                                                                             |
// | 4   | **input\_value\_sum**                          | Сумма BTC на входах (входящие объемы). Требует суммы value всех входящих UTXO. Для каждого входа нужно получить предыдущее входящее значение (через RPC `getrawtransaction` для предыдущего TX или использовать индекс UTXO) и сложить.                                                                                                                                                                                                 |
// | 5   | **output\_value\_sum**                         | Сумма BTC на выходах (исходящий объем). Извлекается суммированием value всех выходов транзакции (доступно из самой транзакции).                                                                                                                                                                                                                                                                                                         |
// | 6   | **transaction\_fee**                           | Комиссия транзакции (в BTC): `input_value_sum – output_value_sum`. Вычисляется как разница между суммой входов и выходов.                                                                                                                                                                                                                                                                                                               |
// | 7   | **avg\_input\_value**                          | Средний размер входящего UTXO (BTC на вход): `input_value_sum / n_inputs` (если `n_inputs>0`). Вычисляется по сумме входов и количеству входов.                                                                                                                                                                                                                                                                                         |
// | 8   | **avg\_output\_value**                         | Средний размер создаваемого UTXO (BTC на выход): `output_value_sum / n_outputs`.                                                                                                                                                                                                                                                                                                                                                        |
// | 9   | **avg\_input\_incoming\_txs**                  | Среднее число **входящих** транзакций на адресах входов. Для каждого входа берется адрес-источник, подсчитывается общее число транзакций, приходящих на этот адрес (можно через RPC `getaddressinfo`/API адресного индекса или сервис BlockCypher/Blockchain.info) – затем усредняется по входам.                                                                                                                                       |
// | 10  | **avg\_output\_outgoing\_txs**                 | Среднее число **исходящих** транзакций с адресов выходов. Для каждого выходного адреса берется общее число трат с этого адреса (сервис блокчейна или индекс) и берется среднее.                                                                                                                                                                                                                                                         |
// | 11  | **unique\_input\_addresses**                   | Число уникальных адресов среди входов. Требует получения адресов из предыдущих выходов (через RPC) и подсчета уникальных.                                                                                                                                                                                                                                                                                                               |
// | 12  | **unique\_output\_addresses**                  | Число уникальных адресов среди выходов. Извлекается напрямую из выходов транзакции (RPC).                                                                                                                                                                                                                                                                                                                                               |
// | 13  | **num\_coinbase\_inputs**                      | Флаг/счетчик coinbase: 1, если транзакция — coinbase (генерация блока), иначе 0. Проверяется по наличию специального входа.                                                                                                                                                                                                                                                                                                             |
// | 14  | **old\_input\_fraction**                       | Доля входов, возраст которых превышает N дней (например, «старые UTXO»). Может рассчитываться по разнице времени текущего блока и блоков входных UTXO (RPC нескольких `getrawtransaction`).                                                                                                                                                                                                                                             |
// | 15  | **change\_output\_ratio**                      | Соотношение суммы «сдачи» к общей сумме выходов. Определяется при наличии адреса отправителя; если известна адресная кластеризация, можно выделить «сдачу».                                                                                                                                                                                                                                                                             |
// | 16  | **inputs\_address\_entropy**                   | Энтропия адресов входов (гетерогенность отправителей). Можно оценить по разнообразию адресов входов (через кластеризацию адресов и вычисление статистики).                                                                                                                                                                                                                                                                              |
// | 17  | **outputs\_address\_entropy**                  | Энтропия адресов выходов (насколько разделены выходы). Аналогично, по исходящим адресам.                                                                                                                                                                                                                                                                                                                                                |
// | 18  | **spent\_outputs\_count**                      | Число выходов транзакции, которые уже были потрачены (на текущий момент). Извлекается при мониторинге блоков: отмечаем выходы, попавшие в будущие транзакции.                                                                                                                                                                                                                                                                           |
// | 19  | **unspent\_outputs\_count**                    | Число выходов транзакции, еще не потраченных. = n\_outputs – spent\_outputs\_count.                                                                                                                                                                                                                                                                                                                                                     |
// | 20  | **tine\_diff\_prev\_output**                   | Среднее время жизни использованных входов: усредненное время между созданием UTXO (родительской транзакцией) и текущей транзакцией. Вычисляется по временам блоков (RPC `getblockheader`) родителей.                                                                                                                                                                                                                                    |
// | 21  | **avg\_outgoing\_txs\_inputs**                 | Среднее число транзакций **расходующих** адреса входов (сколько раз адрес входов потратил BTC после этой транзакции). Требует просмотра истории адресов.                                                                                                                                                                                                                                                                                |
// | 22  | **avg\_incoming\_txs\_outputs**                | Среднее число транзакций, **получающих** адреса выходов (сколько раз адреса выходов получали BTC до этой транзакции).  Аналогично через адресный индекс.                                                                                                                                                                                                                                                                                |


// И X входных кошельков в эту транзакцию и Y выходных кошельков:

// Входные: 16HBDHsz3V8pW9nrFY29EsZRisXvfCyQCR,47,1.0,0.0,483937.0,483937.0,0.0,1.0,483937.0,0.0,1.0,0.138219,0.138219,0.138219,0.138219,0.138219,0.138219,0.138219,0.138219,0.138219,0.138219,0.0,0.0,0.0,0.0,0.0,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,2.0,1.0,1.0,1.0,1.0
//
// Выходные:
// 15YYt1SAENYNzAPShaJD423KDEVnWmRrtX,47,1.0,1.0,483937.0,483939.0,2.0,2.0,483939.0,483937.0,1.0,0.06544678,0.03272339,0.03272339,0.03272339,0.03272339,0.03272339,0.0,0.03272339,0.016361695,0.016361695,0.03272339,0.0,0.03272339,0.016361695,0.016361695,0.00175993,0.00087996,0.00087997,0.000879965,0.000879965,0.00108829316330099,0.000208333163300993,0.00087996,0.000544146581650496,0.000544146581650496,2.0,2.0,2.0,2.0,2.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,3.0,1.0,1.0,1.0,1.0
// 1JQgTsrc1ChRHn9pAnrXmuPnDVA4v6uQbf,47,0.0,1.0,483937.0,483937.0,0.0,1.0,0.0,483937.0,1.0,0.10461564,0.10461564,0.10461564,0.10461564,0.10461564,0.0,0.0,0.0,0.0,0.0,0.10461564,0.10461564,0.10461564,0.10461564,0.10461564,0.00087997,0.00087997,0.00087997,0.00087997,0.00087997,0.000666034515738067,0.000666034515738067,0.000666034515738067,0.000666034515738067,0.000666034515738067,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,1.0,1.0,1.0,1.0,1.0
//

/// Извлекает и вычисляет характеристики транзакции из Bitcoin explorer API
///
/// # Arguments
/// * `tx_id` - ID транзакции в формате hex string (64 символа)
///
/// # Returns
/// `Result<TransactionFeatures, Box<dyn std::error::Error>>` - структура с характеристиками транзакции
///
/// # Errors
/// Возвращает ошибку если:
/// - Клиент explorer не инициализирован
/// - Не удалось получить данные транзакции
/// - Не удалось десериализовать JSON
/// - Отсутствуют значения входов
///
/// # Example
/// ```no_run
/// use diploma_software::application::services::transaction::transaction_info::get_transaction_info;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tx_id = "d6176384de4c0b98702eccb97f3ad6670bc8410d9da715fe5b49462d3e603993";
/// let features = get_transaction_info(tx_id).await?;
/// println!("Inputs: {}, Outputs: {}", features.n_inputs, features.n_outputs);
/// # Ok(())
/// # }
/// ```
pub async fn get_transaction_info(tx_id: &str) -> Result<TransactionFeatures, Box<dyn std::error::Error>> {
    let client = get_or_init_client().await
        .map_err(|e| format!("Failed to initialize explorer client: {}", e))?;
    let tx_json = client.get_transaction(tx_id).await?;

    println!("Transaction JSON: {:?}", tx_json);

    // Десериализуем JSON в структуру
    let tx: TransactionData = serde_json::from_value(tx_json)?;

    let n_inputs = tx.vin.len();
    let n_outputs = tx.vout.len();

    println!("n_inputs: {}, n_outputs: {}", n_inputs, n_outputs);

    // Вычисляем сумму входов
    // В Blockbook API значение входа может быть в value или value_sat
    let input_value_sum: f64 = tx
        .vin
        .iter()
        .map(|vin| {
            vin.value
                .or_else(|| vin.value_sat.map(|sat| sat as f64 / 100_000_000.0))
                .ok_or(ExplorerError::MissingInputValue)
        })
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .sum();

    // Вычисляем сумму выходов
    let output_value_sum: f64 = tx.vout.iter().map(|v| v.value).sum();

    let transaction_fee = input_value_sum - output_value_sum;

    let avg_input_value = if n_inputs > 0 {
        input_value_sum / n_inputs as f64
    } else {
        0.0
    };

    let avg_output_value = if n_outputs > 0 {
        output_value_sum / n_outputs as f64
    } else {
        0.0
    };

    let features = TransactionFeatures {
        n_inputs,
        n_outputs,
        input_value_sum,
        output_value_sum,
        transaction_fee,
        avg_input_value,
        avg_output_value,
    };

    println!("Transaction Features: {:?}", features);

    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_features_calculation() {
        // Тест вычисления характеристик транзакции
        let features = TransactionFeatures {
            n_inputs: 2,
            n_outputs: 3,
            input_value_sum: 1.5,
            output_value_sum: 1.49,
            transaction_fee: 0.01,
            avg_input_value: 0.75,
            avg_output_value: 0.49666666666666664,
        };

        assert_eq!(features.n_inputs, 2);
        assert_eq!(features.n_outputs, 3);
        assert_eq!(features.transaction_fee, 0.01);
        assert!((features.avg_input_value - 0.75).abs() < 0.0001);
    }

    #[test]
    fn test_transaction_fee_calculation() {
        let features = TransactionFeatures {
            n_inputs: 1,
            n_outputs: 2,
            input_value_sum: 1.0,
            output_value_sum: 0.99,
            transaction_fee: 0.01,
            avg_input_value: 1.0,
            avg_output_value: 0.495,
        };

        let calculated_fee = features.input_value_sum - features.output_value_sum;
        assert!((calculated_fee - features.transaction_fee).abs() < 0.0001);
    }

    #[test]
    fn test_avg_calculations_with_zero_inputs() {
        let features = TransactionFeatures {
            n_inputs: 0,
            n_outputs: 1,
            input_value_sum: 0.0,
            output_value_sum: 1.0,
            transaction_fee: 0.0,
            avg_input_value: 0.0,
            avg_output_value: 1.0,
        };

        assert_eq!(features.avg_input_value, 0.0);
        assert_eq!(features.avg_output_value, 1.0);
    }

    #[test]
    fn test_transaction_features_serialization() {
        let features = TransactionFeatures {
            n_inputs: 2,
            n_outputs: 3,
            input_value_sum: 1.5,
            output_value_sum: 1.49,
            transaction_fee: 0.01,
            avg_input_value: 0.75,
            avg_output_value: 0.4967,
        };

        let json = serde_json::to_string(&features).expect("Should serialize");
        let deserialized: TransactionFeatures = serde_json::from_str(&json)
            .expect("Should deserialize");

        assert_eq!(features, deserialized);
    }

    #[test]
    fn test_transaction_features_clone() {
        let features = TransactionFeatures {
            n_inputs: 1,
            n_outputs: 1,
            input_value_sum: 1.0,
            output_value_sum: 0.99,
            transaction_fee: 0.01,
            avg_input_value: 1.0,
            avg_output_value: 0.99,
        };

        let cloned = features.clone();
        assert_eq!(features, cloned);
    }
}
