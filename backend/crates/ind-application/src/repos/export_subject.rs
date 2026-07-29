use uuid::Uuid;

/// Which relationship an export subject is keyed on (TASK-236 AC#4/#5).
///
/// `LibraryEntry` subjects are saved Library content: they are enumerated from
/// `library_entries JOIN documents`, written to / advanced through the `library_entry_id`-keyed
/// cursor/refresh/artifact tables, and carry Library provenance + Library tags.
///
/// `Document` subjects are documents with authored capabilities (a highlight/note, or a
/// retained `user_document_state`) that have no active `library_entries` row. They surface only
/// under [`ExportScope::IncludeUnsavedAuthored`] and are snapshot-exported: they have no
/// `library_entry_id`, so they are never written to the cursor tables (cursor advance is a no-op).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportSubjectKind {
    LibraryEntry,
    Document,
}

impl ExportSubjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LibraryEntry => "library_entry",
            Self::Document => "document",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "library_entry" => Some(Self::LibraryEntry),
            "document" => Some(Self::Document),
            _ => None,
        }
    }

    /// Stable sort rank used in keyset pagination so `LibraryEntry` and `Document` subjects that
    /// share an `effective_changed_at` order deterministically and never skip/duplicate.
    pub fn rank(self) -> i16 {
        match self {
            Self::LibraryEntry => 0,
            Self::Document => 1,
        }
    }
}

/// Stable export subject identity. `id` is the `library_entry_id` for `LibraryEntry` subjects and
/// the `document_id` for `Document` subjects. The wire `subject_id` field (TASK-237) carries the
/// canonical id for that kind: `lib_<id>` for `LibraryEntry`, `doc_<id>` for `Document`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportSubject {
    pub kind: ExportSubjectKind,
    pub id: Uuid,
}

impl ExportSubject {
    pub fn library_entry(id: Uuid) -> Self {
        Self {
            kind: ExportSubjectKind::LibraryEntry,
            id,
        }
    }

    pub fn document(id: Uuid) -> Self {
        Self {
            kind: ExportSubjectKind::Document,
            id,
        }
    }
}

/// Export coverage (TASK-236 AC#4/#5). `LibraryOnly` (default) enumerates only saved Library
/// content. `IncludeUnsavedAuthored` additionally surfaces unsaved-but-authored `Document`
/// subjects, for the explicit "export my annotations outside the Library" operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportScope {
    #[default]
    LibraryOnly,
    IncludeUnsavedAuthored,
}

impl ExportScope {
    pub fn includes_unsaved_authored(self) -> bool {
        matches!(self, Self::IncludeUnsavedAuthored)
    }
}
