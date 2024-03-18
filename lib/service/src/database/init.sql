create table account
(
    id       INTEGER not null,
    account  TEXT    not null constraint account_pk unique,
    password TEXT    not null
);

create table cloud_file
(
    id                 integer constraint table_name_pk primary key autoincrement,
    cloud_id           integer not null,
    file_id            text,
    hash               text,
    file_block_meta_id integer
);

create unique index table_name_id_uindex
    on cloud_file (id);

create table cloud_file_block
(
    id              integer not null constraint id primary key autoincrement,
    file_block_id   integer not null,
    cloud_meta_id   integer,
    cloud_file_id   text,
    cloud_file_hash text,
    status          integer,
    create_time     integer,
    update_time     integer,
    deleted         integer
);

create table cloud_meta
(
    id              integer constraint cloud_meta_pk primary key autoincrement,
    name            text not null,
    token           text,
    last_work_time  datetime default 0,
    data_root       text,
    status          integer,
    deleted         INTEGER,
    cloud_type      int,
    total_quota     INTEGER,
    used_quota      integer,
    remaining_quota integer,
    extra           TEXT,
    expires_in      integer
);

create table config
(
    id          INTEGER not null constraint config_pk primary key autoincrement,
    property    TEXT,
    value       TEXT,
    description text
);

create table file_block_meta
(
    id               integer           not null constraint file_block_pk primary key autoincrement,
    block_index      integer           not null,
    file_part_id     text              not null,
    cloud_file_id    text,
    update_time      integer,
    file_modify_time integer default 0 not null,
    deleted          integer default 0,
    file_meta_id     integer           not null,
    part_hash        text,
    cloud_file_hash  text,
    status           integer
);

create unique index file_block_meta_file_meta_id_block_index_uindex
    on file_block_meta (file_meta_id, block_index);

create table file_meta
(
    id          integer           not null constraint file_meta_pk primary key autoincrement,
    parent_id   integer           not null,
    name        text              not null,
    file_type   integer           not null,
    file_length integer           not null,
    create_time datetime,
    update_time datetime,
    deleted     integer default 0 not null,
    status      integer
);

create index file_meta_parent_id_name_index
    on file_meta (parent_id, name);
