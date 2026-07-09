# Issue #26 -- [packages] Bootstrap `ed.persistence-mongo` -- mongodb driver wrapper

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/persistence-mongo/`:

- `MongoDb { client: mongodb::Client }` connection helper; `init_indexes()` for all collections declared by the consumer.
- `MongoRepo<T: Serialize + DeserializeOwned + Send + Sync>` with helpers `find_one`, `find_many`, `insert_one`, `update_one`, `delete_one`, `soft_delete`, `replace_one`.
- Conventions applied automatically: `_id: ObjectId`, `created_at`, `updated_at`, `is_deleted`, `deleted_at`. Converter trait `BsonFrom<T>` / `BsonInto<T>`.
- Free-form collection support for `room_snapshots` (binary blob) and `latex_artifacts` (metadata).

**Acceptance**
- `cargo test -p ed_persistence_mongo --features test-mongo` runs against `mongo:7` testcontainer.
- `replace_one` preserves `_id` and concurrency-fails with a clear error.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-26.md` recording the work for issue #26.
