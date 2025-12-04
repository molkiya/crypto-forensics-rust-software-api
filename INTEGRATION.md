# Интеграция с Python LLM репозиторием

Этот документ описывает процесс интеграции Rust приложения с Python репозиторием, который разрабатывает LLM для анализа транзакций.

## Архитектура интеграции

```
┌─────────────────┐         ┌──────────────────┐
│  Rust Backend   │ ◄─────► │  Python LLM      │
│  (Web Server)   │  JSON   │  (ML Service)     │
└─────────────────┘         └──────────────────┘
         │                           │
         │                           │
         ▼                           ▼
┌─────────────────┐         ┌──────────────────┐
│  Bitcoin        │         │  ML Models       │
│  Explorer API   │         │  (aml_bitcoin.pth)│
└─────────────────┘         └──────────────────┘
```

## Формат данных

**⚠️ ВАЖНО**: Полная спецификация API и моделей данных находится в файле [PYTHON_API_SPEC.md](PYTHON_API_SPEC.md). Ниже приведена краткая сводка.

### Запрос от Rust к Python

Минимальный формат (текущая реализация):

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
  }
}
```

Полный формат (когда будут реализованы все признаки) включает:
- `transaction_features` - базовые характеристики (7 полей)
- `extended_features` - расширенные характеристики (15 полей)
- `addresses` - списки входных и выходных адресов
- `input_features` - признаки для каждого входного адреса (55 признаков)
- `output_features` - признаки для каждого выходного адреса (55 признаков)

### Ответ от Python к Rust

```json
{
  "success": true,
  "transaction_id": "d6176384de4c0b98702eccb97f3ad6670bc8410d9da715fe5b49462d3e603993",
  "prediction": {
    "class": "illicit",
    "confidence": 0.95,
    "risk_score": 0.87
  },
  "explanation": "Transaction shows patterns consistent with money laundering",
  "details": {
    "model_version": "1.0.0",
    "inference_time_ms": 45.2
  }
}
```

**См. [PYTHON_API_SPEC.md](PYTHON_API_SPEC.md) для полной спецификации всех полей и типов данных.**

## API Endpoints для интеграции

### Предлагаемая структура Python API

- `POST /api/v1/analyze` - Анализ транзакции
- `POST /api/v1/batch_analyze` - Пакетный анализ
- `GET /api/v1/health` - Проверка работоспособности

## Реализация в Rust

Пример интеграции можно добавить в `src/services/ml/`:

```rust
// src/services/ml/mod.rs
pub mod ml_client;

// src/services/ml/ml_client.rs
pub struct MLClient {
    base_url: String,
    client: reqwest::Client,
}

impl MLClient {
    pub async fn analyze_transaction(&self, tx_data: TransactionData) -> Result<MLResponse> {
        // Отправка запроса к Python сервису
    }
}
```

## Конфигурация

Добавить в `.env`:

```bash
# Python ML Service
ML_SERVICE_URL=http://localhost:8001
ML_SERVICE_TIMEOUT=30
```

## Модели

Общие модели ML хранятся в `src/infrastructure/model/`:
- `aml_bitcoin.pth` - Модель для анализа AML (Anti-Money Laundering)

## Тестирование интеграции

1. Запустить Python сервис
2. Запустить Rust приложение
3. Отправить тестовую транзакцию через веб-интерфейс
4. Проверить ответ от ML сервиса

## Примечания

- Используйте асинхронные HTTP запросы через `reqwest`
- Обрабатывайте таймауты и ошибки сети
- Кэшируйте результаты для часто запрашиваемых транзакций
- Логируйте все запросы к ML сервису для отладки

