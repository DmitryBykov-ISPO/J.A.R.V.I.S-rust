# J.A.R.V.I.S (Rust) — форк Bossiara13

Форк Rust-переписки голосового ассистента [Priler/jarvis](https://github.com/Priler/jarvis).
Текущий репозиторий: <https://github.com/DmitryBykov-ISPO/J.A.R.V.I.S-rust>.

## Что это

Голосовой ассистент, написанный на Rust, работающий локально (без облака).
Текущий стек:

- Vosk — Speech-to-Text (через `vosk-rs`).
- fastembed + ort — локальные эмбеддинги для intent-классификации (MiniLM L6/L12 ONNX).
- Picovoice Porcupine / Rustpotter / Vosk — три опциональных движка wake-word.
- mlua (Lua 5.5, vendored) — скрипты пользовательских команд.
- Tauri + Vite/Svelte — GUI-оболочка (фронтенд в отдельной папке `frontend/`).
- nnnoiseless — подавление шума.
- fluent / unic-langid — i18n (`ru`, `ua`, `en`).

**LLM-клиент (Groq / OpenAI-совместимый) добавлен в `jarvis-core::llm` и подключён к голосовому циклу.** Если фраза начинается с триггера («скажи», «ответь», «произнеси»), она уходит в Groq и ответ возвращается через IPC-событие `LlmReply`. Без триггера всё работает как раньше — wake-word + intent + Lua. Это следующий шаг после CLI-only LLM из v0.2.0.

## Это форк

Оригинальный автор — Abraham Tugalov (Priler).
Апстрим: <https://github.com/Priler/jarvis>.
Лицензия сохранена: **CC BY-NC-SA 4.0** (см. `LICENSE.txt`).
Атрибуция в `Cargo.toml` и `voice.toml` пакетов озвучки не изменена.

## Что отличается от апстрима

- Обновлён список авторов в `Cargo.toml` (добавлен `Bossiara13 (fork)`, оригинал сохранён).
- README переписан и отражает фактическую архитектуру (апстримный README называет проект "Tauri+Svelte", что давно не соответствует действительности — это workspace из 4-х крейтов).
- Отсутствующие в апстриме ONNX-модели (`all-MiniLM-L6-v2`, `paraphrase-multilingual-MiniLM-L12-v2-onnx-Q`) подтянуты через Git LFS из HuggingFace (Qdrant) и запушены в форк.

## Структура репозитория

Cargo workspace из четырёх крейтов:

| Крейт          | Назначение                                                                 |
|----------------|----------------------------------------------------------------------------|
| `jarvis-core`  | Библиотека: конфиг, intent, STT, wake-word, аудио, Lua-бэкенд, i18n.       |
| `jarvis-app`   | Бинарь-«демон»: собирает всё вместе, tray, IPC.                             |
| `jarvis-gui`   | Tauri-приложение (использует `frontend/dist/client`).                       |
| `jarvis-cli`   | CLI для отладки: классификация intent, список команд, dump конфига.         |

Прочее:

- `frontend/` — Vite + Svelte UI для `jarvis-gui`. Собирается отдельно.
- `lib/windows/amd64/` — нативные DLL/LIB для Vosk, Porcupine, PvRecorder.
- `resources/` — голоса, модели, конфиги по умолчанию. ONNX-модели хранятся в Git LFS.
- `post_build.py` — постпроцессинг артефактов сборки (Python 3).

## Сборка

Требования:

- Rust 1.93+ (собирается на stable MSVC).
- Node 24+ и npm — для фронтенда.
- Python 3 — для `post_build.py`.
- MSVC build tools (Windows, x64).
- Установленные `libvosk.lib`, `libpv_porcupine.dll`, `libpv_recorder.dll` в `lib/windows/amd64/` (уже в репозитории).

Перед сборкой `jarvis-gui` нужно собрать фронтенд:

```bash
cd frontend
npm install
npm run build
cd ..
```

Затем workspace:

```bash
cargo build --workspace
```

Холодная сборка занимает около 10 минут (ONNX runtime, aws-lc-rs, tauri).

## Статус сборки в этом форке

На моей машине (`cargo build --workspace`, stable MSVC) итог:

- `jarvis-core` — собрался (1 warning, unused import).
- `jarvis-app` — собрался, бинарник `target/debug/jarvis-app.exe` создан.
- `jarvis-cli` — **падает на линковке**: `LNK1181: cannot open input file "libvosk.lib"`.
  Причина: у `jarvis-cli` нет своего `build.rs`, а `.cargo/config.toml` с `rustc-link-search` лежит только внутри `crates/jarvis-app/` и не подтягивается для `jarvis-cli`. Лечится либо добавлением такого же `build.rs` в `crates/jarvis-cli/`, либо вынесением `config.toml` в корень. Сознательно не трогал — фикс выходит за рамки рефакторинга (v0.0.1-import фиксирует поведение апстрима как есть).
- `jarvis-gui` — падает в `tauri::generate_context!()`: `frontendDist = "../../frontend/dist/client"` не существует. Это ожидаемо, если не запустить `npm run build` в `frontend/` заранее (см. секцию «Сборка»).

Запуск уже собранного:

```bash
./target/debug/jarvis-app.exe
```

Для CLI (`jarvis-cli --help`, команды `classify`, `execute`, `list`, `phrases`) нужно сначала починить линковку Vosk (см. выше).

## LLM (Groq)

В `jarvis-core` есть модуль `llm` — блокирующий клиент для OpenAI-совместимого эндпоинта chat completions. По умолчанию настроен на Groq. Используется через фиче-флаг `llm` (включён в дефолтный набор `jarvis_app`, также подтянут в `jarvis-cli`).

Переменные окружения:

| Переменная      | Обязательна | Значение по умолчанию                  |
|-----------------|-------------|----------------------------------------|
| `GROQ_TOKEN`    | да          | —                                      |
| `GROQ_BASE_URL` | нет         | `https://api.groq.com/openai/v1`       |
| `GROQ_MODEL`    | нет         | `llama-3.3-70b-versatile`              |

Быстрая проверка через CLI:

```bash
set GROQ_TOKEN=gsk_...
jarvis-cli ask "скажи привет одной фразой"
```

Ответ печатается в stdout. Без `GROQ_TOKEN` команда завершится с кодом 2 и сообщением об ошибке. При ошибке API — код 1 и тело ответа.

Программное использование из Rust:

```rust
use jarvis_core::llm::{LlmClient, ChatMessage};

let client = LlmClient::from_env()?;
let reply = client.complete(&[ChatMessage::user("привет")], 256)?;
println!("{}", reply);
```

### Подключение к голосовому циклу

Помимо CLI, LLM подключён напрямую в `jarvis-app`. Логика в `crates/jarvis-app/src/llm_fallback.rs`:

- При старте `jarvis-app` пытается прочитать `GROQ_TOKEN`. Если переменной нет — фоллбэк отключается, в лог пишется warning, голосовые команды продолжают работать как раньше.
- Распознанная фраза (как из микрофона, так и из текстовой панели GUI) проверяется на префиксы-триггеры из `config::get_llm_trigger_phrases` (для `ru`/`ua`: `скажи`, `ответь`, `произнеси`; для `en`: `say`, `tell`, `answer`).
- Если триггер найден — остаток фразы уходит в `LlmClient::complete()`, ответ публикуется в IPC как `IpcEvent::LlmReply { text }` (UI/GUI слушает этот ивент и проговаривает текст уже на своей стороне), а звук-«ок» проигрывается из текущего голосового пресета.
- История разговора хранится в `ConversationHistory` с потолком `LLM_DEFAULT_MAX_HISTORY = 8` ходов; system-prompt всегда сохраняется при вытеснении старых ходов.
- При сетевой/API-ошибке последний user-turn убирается из истории, в IPC уходит `LlmReply` с короткой русской фразой («Не могу связаться с сервером, сэр.»), играется звук-«error». Голосовой цикл не падает.

Системный промпт (русский) описывает J.A.R.V.I.S. как британского дворецкого Тони Старка — короткие реплики (1–3 предложения), обращение «сэр», без излишней цензуры. Меняется в `config::LLM_SYSTEM_PROMPT_RU`.

## Лицензия

Creative Commons **Attribution-NonCommercial-ShareAlike 4.0 International** (CC BY-NC-SA 4.0).
Полный текст — в `LICENSE.txt`. Атрибуция оригинального автора (Abraham Tugalov) сохранена.

В `Cargo.toml` декларирован `license = "GPL-3.0-only"` — это несоответствие унаследовано от апстрима и не правилось, чтобы не расходиться с upstream-конфигом. Приоритет имеет `LICENSE.txt`.

## Python-версия

Старая версия ассистента была на Python.
Последний коммит с Python-кодом в апстриме — [943efbf](https://github.com/Priler/jarvis/tree/943efbfbdb8aeb5889fa5e2dc7348ca4ea0b81df).
