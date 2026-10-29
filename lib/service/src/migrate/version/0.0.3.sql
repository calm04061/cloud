update file_meta
set deleted = 1, update_time = datetime()
where deleted = 0
  and id not in (
      select min(id)
      from file_meta
      where deleted = 0
      group by parent_id, name
  );

update cloud_file_block
set deleted = 1, update_time = datetime()
where deleted = 0
  and file_block_id in (
      select id
      from (
          select id, row_number() over (
              partition by file_meta_id, block_index
              order by part_hash is null, id
          ) as rn
          from file_block_meta
          where deleted = 0
      )
      where rn > 1
  );

update file_block_meta
set deleted = 1, update_time = datetime()
where deleted = 0
  and id in (
      select id
      from (
          select id, row_number() over (
              partition by file_meta_id, block_index
              order by part_hash is null, id
          ) as rn
          from file_block_meta
          where deleted = 0
      )
      where rn > 1
  );

update cloud_file_block
set deleted = 1, update_time = datetime()
where deleted = 0
  and id in (
      select id
      from (
          select id, row_number() over (
              partition by file_block_id, cloud_meta_id
              order by cloud_file_id is null, id
          ) as rn
          from cloud_file_block
          where deleted = 0
      )
      where rn > 1
  );

create unique index if not exists unq_file_meta_parent_name_live
    on file_meta (parent_id, name) where deleted = 0;

create unique index if not exists unq_file_block_meta_file_index_live
    on file_block_meta (file_meta_id, block_index) where deleted = 0;

create unique index if not exists unq_cloud_file_block_live
    on cloud_file_block (file_block_id, cloud_meta_id) where deleted = 0;
