//! File browsing and transfer.

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// FileListData is sent from hub to agent to list a directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileListData {
    pub request_id: String,
    pub path: String,
    #[serde(skip_serializing_if = "is_default")]
    pub show_hidden: bool,
}

/// FileEntry describes a single file/directory in a listing or search result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileEntry {
    pub name: String,
    /// absolute path; populated in search results
    #[serde(skip_serializing_if = "is_default")]
    pub path: String,
    pub size: i64,
    pub mode: String,
    pub mod_time: String,
    pub is_dir: bool,
}

/// FileListedData is sent from agent to hub with directory listing results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileListedData {
    pub request_id: String,
    pub path: String,
    #[serde(deserialize_with = "null_default")]
    pub entries: Vec<FileEntry>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// FileReadData is sent from hub to agent to read a file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileReadData {
    pub request_id: String,
    pub path: String,
}

/// FileDataPayload carries file content chunks (base64-encoded).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileDataPayload {
    pub request_id: String,
    /// base64-encoded chunk
    pub data: String,
    /// byte offset
    pub offset: i64,
    /// true if this is the last chunk
    pub done: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// FileWriteData is sent from hub to agent to write a file chunk.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileWriteData {
    pub request_id: String,
    pub path: String,
    /// base64-encoded chunk
    pub data: String,
    /// byte offset
    pub offset: i64,
    /// true if this is the last chunk
    pub done: bool,
}

/// FileWrittenData is sent from agent to hub confirming write.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileWrittenData {
    pub request_id: String,
    pub bytes_written: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// FileMkdirData is sent from hub to agent to create a directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileMkdirData {
    pub request_id: String,
    pub path: String,
}

/// FileDeleteData is sent from hub to agent to delete a file or directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileDeleteData {
    pub request_id: String,
    pub path: String,
}

/// FileRenameData is sent from hub to agent to rename/move a file or directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileRenameData {
    pub request_id: String,
    pub old_path: String,
    pub new_path: String,
}

/// FileCopyData is sent from hub to agent to copy a file or directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileCopyData {
    pub request_id: String,
    pub src_path: String,
    pub dst_path: String,
}

/// FileResultData is a generic result for file operations (mkdir, delete, rename, copy).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileResultData {
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// FileSearchData is sent from hub to agent to search for files matching a pattern.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileSearchData {
    pub request_id: String,
    pub path: String,
    /// glob pattern matched against filename (e.g. "*.log")
    pub pattern: String,
    /// default 100, capped at 500
    pub max_results: i64,
}

/// FileSearchResultData is sent from agent to hub with the filename search results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileSearchResultData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub matches: Vec<FileEntry>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
    /// true when results were capped at MaxResults
    pub truncated: bool,
}
