/// Findex engine library.
/// Fast Windows file search using file system indexing.

pub mod index_engine;
pub mod pinyin;
pub mod searcher;
pub mod storage;
pub mod types;
pub mod walker;

pub use index_engine::TrieIndex;
pub use searcher::Searcher;
pub use storage::Storage;
pub use types::*;
pub use walker::FsWalker;
