CREATE INDEX "main"."idx_file_block_id_hash"
    ON "cloud_file_block" (
                           "file_block_id",
                           "cloud_file_hash"
        );

CREATE INDEX "main"."idx_status_exp"
    ON "cloud_meta" (
                     "status",
                     "deleted",
                     "expires_in"
        );

CREATE UNIQUE INDEX "main"."unq_property"
    ON "config" (
                 "property"
        );

