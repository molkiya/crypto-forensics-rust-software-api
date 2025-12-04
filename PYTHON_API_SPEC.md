# Спецификация API для Python ML сервиса

Этот документ описывает точную модель данных, которую Rust приложение отправляет Python ML сервису и ожидает получить в ответ.

## Формат запроса (Rust → Python)

### Endpoint: `POST /api/v1/analyze`

Rust приложение отправляет JSON с полной информацией о транзакции:

```json
{
  "transaction_id": "d6176384de4c0b98702eccb97f3ad6670bc8410d9da715fe5b49462d3e603993",
  "transaction_features": {
    "n_inputs": 2,
    "n_outputs": 3,
    "input_value_sum": 1.5,
    "output_value_sum": 1.49,
    "transaction_fee": 0.01,
    "avg_input_value": 0.75,
    "avg_output_value": 0.4967
  },
  "extended_features": {
    "time_step": 47,
    "avg_input_incoming_txs": 5.2,
    "avg_output_outgoing_txs": 3.1,
    "unique_input_addresses": 2,
    "unique_output_addresses": 3,
    "num_coinbase_inputs": 0,
    "old_input_fraction": 0.3,
    "change_output_ratio": 0.15,
    "inputs_address_entropy": 0.85,
    "outputs_address_entropy": 0.92,
    "spent_outputs_count": 1,
    "unspent_outputs_count": 2,
    "time_diff_prev_output": 86400.0,
    "avg_outgoing_txs_inputs": 2.5,
    "avg_incoming_txs_outputs": 1.8
  },
  "addresses": {
    "inputs": [
      "16HBDHsz3V8pW9nrFY29EsZRisXvfCyQCR",
      "1ABC..."
    ],
    "outputs": [
      "15YYt1SAENYNzAPShaJD423KDEVnWmRrtX",
      "1JQgTsrc1ChRHn9pAnrXmuPnDVA4v6uQbf",
      "1XYZ..."
    ]
  },
  "input_features": [
    {
      "address": "16HBDHsz3V8pW9nrFY29EsZRisXvfCyQCR",
      "features": [47, 1.0, 0.0, 483937.0, 483937.0, 0.0, 1.0, 483937.0, 0.0, 1.0, 0.138219, ...]
    }
  ],
  "output_features": [
    {
      "address": "15YYt1SAENYNzAPShaJD423KDEVnWmRrtX",
      "features": [47, 1.0, 1.0, 483937.0, 483939.0, 2.0, 2.0, 483939.0, 483937.0, 1.0, 0.06544678, ...]
    },
    {
      "address": "1JQgTsrc1ChRHn9pAnrXmuPnDVA4v6uQbf",
      "features": [47, 0.0, 1.0, 483937.0, 483937.0, 0.0, 1.0, 0.0, 483937.0, 1.0, 0.10461564, ...]
    }
  ]
}
```

### Структура данных запроса

#### Базовые характеристики транзакции (`transaction_features`)

| Поле | Тип | Описание |
|------|-----|----------|
| `n_inputs` | `usize` | Число входов транзакции (количество UTXO, используемых как входы) |
| `n_outputs` | `usize` | Число выходов транзакции (сколько UTXO создается) |
| `input_value_sum` | `f64` | Сумма BTC на входах (входящие объемы) |
| `output_value_sum` | `f64` | Сумма BTC на выходах (исходящий объем) |
| `transaction_fee` | `f64` | Комиссия транзакции (в BTC): `input_value_sum – output_value_sum` |
| `avg_input_value` | `f64` | Средний размер входящего UTXO (BTC на вход): `input_value_sum / n_inputs` |
| `avg_output_value` | `f64` | Средний размер создаваемого UTXO (BTC на выход): `output_value_sum / n_outputs` |

#### Расширенные характеристики (`extended_features`)

| Поле | Тип | Описание |
|------|-----|----------|
| `time_step` | `u8` | Порядковый номер «временного шага» транзакции (1–49), соответствующий ее времени (интервал ~2 недели) |
| `avg_input_incoming_txs` | `f64` | Среднее число **входящих** транзакций на адресах входов |
| `avg_output_outgoing_txs` | `f64` | Среднее число **исходящих** транзакций с адресов выходов |
| `unique_input_addresses` | `usize` | Число уникальных адресов среди входов |
| `unique_output_addresses` | `usize` | Число уникальных адресов среди выходов |
| `num_coinbase_inputs` | `u8` | Флаг/счетчик coinbase: 1, если транзакция — coinbase (генерация блока), иначе 0 |
| `old_input_fraction` | `f64` | Доля входов, возраст которых превышает N дней (например, «старые UTXO») |
| `change_output_ratio` | `f64` | Соотношение суммы «сдачи» к общей сумме выходов |
| `inputs_address_entropy` | `f64` | Энтропия адресов входов (гетерогенность отправителей) |
| `outputs_address_entropy` | `f64` | Энтропия адресов выходов (насколько разделены выходы) |
| `spent_outputs_count` | `usize` | Число выходов транзакции, которые уже были потрачены (на текущий момент) |
| `unspent_outputs_count` | `usize` | Число выходов транзакции, еще не потраченных |
| `time_diff_prev_output` | `f64` | Среднее время жизни использованных входов (в секундах) |
| `avg_outgoing_txs_inputs` | `f64` | Среднее число транзакций **расходующих** адреса входов |
| `avg_incoming_txs_outputs` | `f64` | Среднее число транзакций, **получающих** адреса выходов |

#### Признаки входных адресов (`input_features`)

Каждый элемент массива содержит:
- `address`: `String` - Bitcoin адрес
- `features`: `Vec<f64>` - Массив из 55 признаков адреса (формат из комментариев в коде)

Пример формата признаков адреса:
```
[time_step, n_inputs, n_outputs, input_value_sum, output_value_sum, 
 transaction_fee, avg_input_value, avg_output_value, ... (55 признаков)]
```

#### Признаки выходных адресов (`output_features`)

Аналогично входным адресам, каждый элемент содержит:
- `address`: `String` - Bitcoin адрес
- `features`: `Vec<f64>` - Массив из 55 признаков адреса

## Формат ответа (Python → Rust)

### Успешный ответ

```json
{
  "success": true,
  "transaction_id": "d6176384de4c0b98702eccb97f3ad6670bc8410d9da715fe5b49462d3e603993",
  "prediction": {
    "class": "illicit",
    "confidence": 0.95,
    "risk_score": 0.87
  },
  "explanation": "Transaction shows patterns consistent with money laundering: high input/output ratio, multiple mixing addresses detected",
  "details": {
    "model_version": "1.0.0",
    "inference_time_ms": 45.2,
    "feature_importance": {
      "transaction_fee": 0.15,
      "unique_output_addresses": 0.12,
      "outputs_address_entropy": 0.10
    }
  }
}
```

### Структура ответа

| Поле | Тип | Описание |
|------|-----|----------|
| `success` | `bool` | Успешность обработки запроса |
| `transaction_id` | `String` | ID транзакции (эхо запроса) |
| `prediction.class` | `String` | Класс транзакции: `"illicit"`, `"licit"`, `"unknown"` |
| `prediction.confidence` | `f64` | Уверенность модели (0.0 - 1.0) |
| `prediction.risk_score` | `f64` | Оценка риска (0.0 - 1.0) |
| `explanation` | `String` | Текстовое объяснение предсказания |
| `details.model_version` | `String` | Версия модели ML |
| `details.inference_time_ms` | `f64` | Время инференса в миллисекундах |
| `details.feature_importance` | `Object` | Важность признаков (опционально) |

### Ответ с ошибкой

```json
{
  "success": false,
  "error": {
    "code": "INVALID_INPUT",
    "message": "Missing required field: transaction_features",
    "details": {}
  }
}
```

## Rust структуры данных

Для справки, вот соответствующие Rust структуры:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFeatures {
    pub n_inputs: usize,
    pub n_outputs: usize,
    pub input_value_sum: f64,
    pub output_value_sum: f64,
    pub transaction_fee: f64,
    pub avg_input_value: f64,
    pub avg_output_value: f64,
}

#[derive(Debug, Serialize)]
pub struct MLRequest {
    pub transaction_id: String,
    pub transaction_features: TransactionFeatures,
    pub extended_features: ExtendedFeatures,
    pub addresses: Addresses,
    pub input_features: Vec<AddressFeatures>,
    pub output_features: Vec<AddressFeatures>,
}

#[derive(Debug, Deserialize)]
pub struct MLResponse {
    pub success: bool,
    pub transaction_id: String,
    pub prediction: Prediction,
    pub explanation: String,
    pub details: ResponseDetails,
}
```

## Примеры использования

### Минимальный запрос (только базовые признаки)

Если расширенные признаки еще не реализованы, Python сервис должен принимать:

```json
{
  "transaction_id": "...",
  "transaction_features": {
    "n_inputs": 2,
    "n_outputs": 3,
    "input_value_sum": 1.5,
    "output_value_sum": 1.49,
    "transaction_fee": 0.01,
    "avg_input_value": 0.75,
    "avg_output_value": 0.4967
  }
}
```

### Полный запрос (все признаки)

Когда все признаки будут реализованы, используется полный формат из раздела выше.

## Примечания для разработчиков Python сервиса

1. **Обратная совместимость**: Сервис должен работать с минимальным набором признаков (`transaction_features`), даже если расширенные признаки отсутствуют.

2. **Типы данных**:
   - Все числовые значения могут быть `null` (опциональные поля)
   - Массивы могут быть пустыми
   - Строки всегда присутствуют (не `null`)

3. **Валидация**: Python сервис должен валидировать:
   - Наличие обязательных полей (`transaction_id`, `transaction_features`)
   - Корректность типов данных
   - Диапазоны значений (например, `confidence` в [0.0, 1.0])

4. **Производительность**: 
   - Ожидаемое время ответа: < 100ms для одной транзакции
   - Поддержка пакетной обработки через `/api/v1/batch_analyze`

5. **Обработка ошибок**: Все ошибки должны возвращаться в формате JSON с полем `success: false`.

