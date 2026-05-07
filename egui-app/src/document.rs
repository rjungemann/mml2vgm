use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileStatus {
    Idle,
    Compiling,
    Ok { warnings: usize },
    Errors(Vec<CompileError>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
}

pub struct Document {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub content: String,
    pub modified: bool,
    pub compile_status: CompileStatus,
    /// Bytes of the last successful compile output.
    pub compiled_bytes: Option<Vec<u8>>,
    /// Pending auto-compile: time (via egui's time) when content was last changed.
    pub last_edit_time: Option<f64>,
    /// If set, the editor will scroll to and place the cursor at this 1-based line on the next frame.
    pub jump_to_line: Option<usize>,
}

impl Document {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            path: None,
            content: String::new(),
            modified: false,
            compile_status: CompileStatus::Idle,
            compiled_bytes: None,
            last_edit_time: None,
            jump_to_line: None,
        }
    }

    pub fn from_file(id: usize, path: PathBuf, content: String) -> Self {
        Self {
            id,
            path: Some(path),
            content,
            modified: false,
            compile_status: CompileStatus::Idle,
            compiled_bytes: None,
            last_edit_time: None,
            jump_to_line: None,
        }
    }

    pub fn tab_label(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        if self.modified {
            format!("● {name}")
        } else {
            name
        }
    }

    pub fn mark_edited(&mut self, now: f64) {
        self.modified = true;
        self.last_edit_time = Some(now);
    }
}

pub struct DocumentStore {
    docs: Vec<Document>,
    active: Option<usize>,
    next_id: usize,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            active: None,
            next_id: 1,
        }
    }

    pub fn open_untitled(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.docs.push(Document::new(id));
        self.active = Some(id);
        id
    }

    pub fn open_file(&mut self, path: PathBuf, content: String) -> usize {
        // Activate existing tab if already open.
        if let Some(doc) = self.docs.iter().find(|d| d.path.as_ref() == Some(&path)) {
            let id = doc.id;
            self.active = Some(id);
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.docs.push(Document::from_file(id, path, content));
        self.active = Some(id);
        id
    }

    pub fn close(&mut self, id: usize) {
        self.docs.retain(|d| d.id != id);
        if self.active == Some(id) {
            self.active = self.docs.last().map(|d| d.id);
        }
    }

    pub fn active(&self) -> Option<&Document> {
        self.active.and_then(|id| self.docs.iter().find(|d| d.id == id))
    }

    pub fn active_mut(&mut self) -> Option<&mut Document> {
        self.active.and_then(|id| self.docs.iter_mut().find(|d| d.id == id))
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Document> {
        self.docs.iter_mut().find(|d| d.id == id)
    }

    pub fn docs(&self) -> &[Document] {
        &self.docs
    }

    pub fn set_active(&mut self, id: usize) {
        if self.docs.iter().any(|d| d.id == id) {
            self.active = Some(id);
        }
    }

    pub fn active_id(&self) -> Option<usize> {
        self.active
    }

    pub fn has_any(&self) -> bool {
        !self.docs.is_empty()
    }
}
