/// CLI command handlers - Single Responsibility Principle
/// Each handler has one reason to change: its specific command logic

pub mod query;
pub mod list;
pub mod search;
pub mod get;

pub use query::QueryCommand;
pub use list::ListCommand;
pub use search::SearchCommand;
pub use get::GetCommand;
