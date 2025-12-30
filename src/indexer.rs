use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use crate::conf::Config;
use jwalk::WalkDir;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument, Term};
use tracing::{debug, error, info};

const MEMORY_BUDGET: usize = 50_000_000; // 50MB
const COMMIT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct FileData {
    pub mime: Option<String>,
    pub path: Option<PathBuf>,
    pub uri: String,
    pub score: f32,
}

impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for FileData {}

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for FileData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl FileData {
    pub fn try_open(&self) -> bool {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(&self.uri);

        match cmd.spawn() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[inline]
    pub fn icon(&self) -> gtk::gio::Icon {
        let mut icon_name = "text-x-preview";
        if let Some(mime) = &self.mime {
            let attempt = mime.replacen("/", "-", 1);
            icon_name = attempt.as_str();
            crate::icons::get_icon(icon_name)
        } else {
            crate::icons::get_icon(icon_name)
        }
    }
}

pub enum IndexerMsg {
    Search {
        query: String,
        reply: Sender<Vec<FileData>>,
    },
    FileChanged(PathBuf),
    Tick,
    Stop,
}

struct InnerIndexer {
    index: Index,
    writer: IndexWriter,
    filename_field: Field,
    path_field: Field,
    config: Config,
}

impl InnerIndexer {
    fn new(index_dir: &Path, config: Config) -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();

        let filename_field = schema_builder.add_text_field("filename", TEXT | STORED);
        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let _body_field = schema_builder.add_text_field("body", TEXT);

        let schema = schema_builder.build();

        let index = if index_dir.exists() {
            Index::open_in_dir(index_dir)
                .or_else(|_| Index::create_in_dir(index_dir, schema.clone()))?
        } else {
            fs::create_dir_all(index_dir)?;
            Index::create_in_dir(index_dir, schema.clone())?
        };

        let writer = index.writer(MEMORY_BUDGET)?;

        Ok(Self {
            index,
            writer,
            filename_field,
            path_field,
            config,
        })
    }

    fn index_file(&mut self, path: &Path) {
        if !should_index(path, &self.config) {
            return;
        }

        let path_str = path.to_string_lossy().to_string();
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Remove existing document for this path to avoid duplicates (update)
        let term = Term::from_field_text(self.path_field, &path_str);
        self.writer.delete_term(term);

        let mut doc = TantivyDocument::default();
        doc.add_text(self.filename_field, &filename);
        doc.add_text(self.path_field, &path_str);

        if let Err(e) = self.writer.add_document(doc) {
            error!("Failed to add document: {}", e);
        }
    }

    fn commit(&mut self) {
        if let Err(e) = self.writer.commit() {
            error!("Failed to commit index: {}", e);
        }
    }

    fn search(&self, query_str: &str) -> Vec<FileData> {
        let reader = match self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to get index reader: {}", e);
                return vec![];
            }
        };

        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.filename_field]);

        // Simple query parsing
        let query = match query_parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return vec![], // Handle invalid queries gracefully
        };

        let top_docs = match searcher.search(&query, &TopDocs::with_limit(20)) {
            Ok(d) => d,
            Err(e) => {
                error!("Search error: {}", e);
                return vec![];
            }
        };

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                let path_str = retrieved_doc
                    .get_first(self.path_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let path = PathBuf::from(&path_str);
                let mime = mime_guess::from_path(&path).first().map(|m| m.to_string());
                let uri = format!("file://{}", path_str);

                results.push(FileData {
                    mime,
                    path: Some(path),
                    uri,
                    score,
                });
            }
        }
        // Explicitly sort just in case, though TopDocs guarantees it.
        // Reverse because we want highest score first.
        results.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
        results
    }
}

fn should_index(path: &Path, config: &Config) -> bool {
    // Ignore hidden files and directories
    if path
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
    {
        return false;
    }
    // Ignore specific directories
    if path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        // Default exclusions
        if s == "node_modules" || s == "target" || s == "venv" || s == "dist" || s == "build" {
            return true;
        }

        config
            .file_indexer
            .exclude_directories
            .iter()
            .any(|ex| s == *ex)
    }) {
        return false;
    }
    true
}

fn perform_cold_scan(tx: Sender<IndexerMsg>, config: Config) {
    let mut roots = Vec::new();

    if !config.file_indexer.include_directories.is_empty() {
        for dir_str in &config.file_indexer.include_directories {
            let p = PathBuf::from(dir_str);
            if p.exists() {
                roots.push(p);
            }
        }
    } else {
        // Default to home directory if no specific includes
        roots.push(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));
    }

    info!("Starting cold scan on {:?}", roots);

    for root in roots {
        // Configure jwalk
        for entry in WalkDir::new(root).skip_hidden(true).follow_links(false) {
            match entry {
                Ok(dir_entry) => {
                    let path = dir_entry.path();
                    if dir_entry.file_type().is_file() && should_index(&path, &config) {
                        let _ = tx.send(IndexerMsg::FileChanged(path));
                    }
                }
                Err(e) => debug!("Walk error: {}", e),
            }
        }
    }
    info!("Cold scan complete");
}

fn get_index_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("seekr");
    path.push("index");
    path
}

pub fn spawn_indexer(config: Config) -> Sender<IndexerMsg> {
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    let config_clone = config.clone();

    thread::spawn(move || {
        let index_dir = get_index_dir();
        let mut indexer = match InnerIndexer::new(&index_dir, config.clone()) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to create indexer at {:?}: {}", index_dir, e);
                return;
            }
        };

        // Start Watcher
        let tx_watcher = tx_clone.clone();
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    for event in events {
                        let _ = tx_watcher.send(IndexerMsg::FileChanged(event.path));
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            },
        )
        .expect("Failed to create watcher");

        // Watch Documents, Downloads, Pictures, etc., OR custom included dirs
        let dirs_to_watch: Vec<PathBuf> = if !config.file_indexer.include_directories.is_empty() {
            config
                .file_indexer
                .include_directories
                .iter()
                .map(PathBuf::from)
                .collect()
        } else {
            vec![
                dirs::document_dir(),
                dirs::download_dir(),
                dirs::picture_dir(),
                dirs::video_dir(),
                dirs::audio_dir(),
            ]
            .into_iter()
            .flatten()
            .collect()
        };

        for dir in dirs_to_watch {
            if dir.exists() {
                let _ = debouncer.watcher().watch(&dir, RecursiveMode::Recursive);
            }
        }

        // Start Cold Scan
        let tx_crawler = tx_clone.clone();
        thread::spawn(move || {
            perform_cold_scan(tx_crawler.clone(), config_clone);
            // Trigger a commit after scan
            let _ = tx_crawler.send(IndexerMsg::Tick);
        });

        // Ticker
        let tx_tick = tx_clone.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(COMMIT_INTERVAL);
                if tx_tick.send(IndexerMsg::Tick).is_err() {
                    break;
                }
            }
        });

        let mut dirty = false;

        while let Ok(msg) = rx.recv() {
            match msg {
                IndexerMsg::Search { query, reply } => {
                    if dirty {
                        indexer.commit();
                        dirty = false;
                    }
                    let results = indexer.search(&query);
                    let _ = reply.send(results);
                }
                IndexerMsg::FileChanged(path) => {
                    indexer.index_file(&path);
                    dirty = true;
                }
                IndexerMsg::Tick => {
                    if dirty {
                        indexer.commit();
                        dirty = false;
                    }
                }
                IndexerMsg::Stop => {
                    indexer.commit();
                    break;
                }
            }
        }
    });

    tx
}
