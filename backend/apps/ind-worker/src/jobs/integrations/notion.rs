mod auth;
mod export_document;
mod managed_target;
mod support;
mod sync_connection;

pub use export_document::handle_export_document;
#[cfg(any(test, feature = "test-helpers"))]
#[allow(unused_imports)]
pub use export_document::handle_export_document_with_test_highlight_batch_size;
pub use sync_connection::handle_sync_connection;
#[cfg(any(test, feature = "test-helpers"))]
#[allow(unused_imports)]
pub use sync_connection::handle_sync_connection_with_test_item_batch_size;
