use std::collections::HashSet;
use std::io::Write as _;

use ind_domain::{
    DocumentId, DocumentOriginType, DocumentType, GenericJobEnvelope, ImportJobId, ImportMethod,
    ImportSource, IntegrationConnectionId, JobOutboxId, LibraryEntryId, UserId,
    deterministic_origin_id,
};
use ind_test_support::{SavedDocumentFactory, TestDb, UserFactory};
use ind_worker::context::IntegrationJobDeps;
use ind_worker::jobs::integrations;

type CsvRow<'a> = (&'a str, &'a str, &'a str, &'a str, f32, &'a str, &'a str);

struct ReadwiseScenario {
    db: TestDb,
    ctx: IntegrationJobDeps,
    user_id: UserId,
}

impl ReadwiseScenario {
    async fn new() -> Self {
        let db = TestDb::new().await;
        let ctx = common::build_worker_ctx(&db).await.integration_jobs();
        let user_id = UserFactory::new()
            .with_email_verified(true)
            .insert(db.pool())
            .await
            .id;
        Self { db, ctx, user_id }
    }

    async fn import(
        &self,
        csv: Option<&[u8]>,
        archive: Option<&[u8]>,
        opml: Option<&[u8]>,
    ) -> ImportJobId {
        let job = self
            .ctx
            .import_job_repo
            .create(
                self.user_id,
                ImportSource::ReadwiseImport,
                ImportMethod::Zip,
                None,
            )
            .await
            .expect("create import job");
        let base = format!(
            "imports/{}/{}",
            self.user_id.into_uuid(),
            job.id.into_uuid()
        );
        let storage = self.ctx.object_storage.as_ref().expect("object storage");
        let mut keys = serde_json::Map::new();
        for (name, suffix, content_type, bytes) in [
            ("csv_key", "library.csv", "text/csv", csv),
            ("zip_key", "archive.zip", "application/zip", archive),
            ("opml_key", "feeds.opml", "application/xml", opml),
        ] {
            if let Some(bytes) = bytes {
                let key = format!("{base}/{suffix}");
                storage
                    .upload(&key, content_type, bytes::Bytes::copy_from_slice(bytes))
                    .await
                    .expect("upload import artifact");
                keys.insert(name.into(), serde_json::Value::String(key));
            }
        }
        self.ctx
            .import_job_repo
            .set_raw_artifact_key(
                self.user_id,
                job.id,
                serde_json::Value::Object(keys).to_string(),
            )
            .await
            .expect("set artifact keys");
        integrations::dispatch_envelope(
            &self.ctx,
            GenericJobEnvelope {
                outbox_id: JobOutboxId::new(),
                job_type: "import.readwise".into(),
                payload: serde_json::json!({ "import_job_id": job.id.to_string() }),
                dedupe_key: None,
            },
        )
        .await
        .expect("dispatch Readwise import");
        job.id
    }

    async fn document_for_origin(&self, external_id: &str) -> DocumentId {
        let origin_id = deterministic_origin_id(
            DocumentOriginType::ReadwiseImportItem,
            self.user_id,
            &format!("readwise:{external_id}"),
        );
        let id: uuid::Uuid = sqlx::query_scalar(
            "SELECT document_id FROM document_origins \
             WHERE user_id = $1 AND origin_type = 'readwise_import_item' AND origin_id = $2",
        )
        .bind(self.user_id.into_uuid())
        .bind(origin_id)
        .fetch_one(self.db.pool())
        .await
        .expect("document origin");
        DocumentId::from_uuid(id)
    }
}

fn csv(rows: &[CsvRow<'_>]) -> Vec<u8> {
    let mut bytes =
        b"Title,URL,ID,Document tags,Saved date,Reading progress,Location,Seen\n".to_vec();
    for (title, url, id, tags, progress, location, seen) in rows {
        writeln!(
            bytes,
            "{title},{url},{id},\"{tags}\",2024-04-01 00:00:00+00:00,{progress},{location},{seen}"
        )
        .unwrap();
    }
    bytes
}

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (path, bytes) in files {
        zip.start_file(path, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

async fn job_counts(s: &ReadwiseScenario, job_id: ImportJobId) -> (i32, i32, i32) {
    sqlx::query_as(
        "SELECT imported_count, duplicate_count, failed_count FROM import_jobs WHERE id = $1",
    )
    .bind(job_id.into_uuid())
    .fetch_one(s.db.pool())
    .await
    .unwrap()
}
