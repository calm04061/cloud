create table migrate
(
    id      integer not null constraint migrate_pk primary key autoincrement,
    version text
);

create table cloud_meta
(
    id              INTEGER primary key autoincrement,
    name            TEXT not null,
    auth            TEXT,
    last_work_time  int8,
    data_root       TEXT,
    status          int  not null,
    deleted         int  not null,
    cloud_type      int  not null,
    total_quota     int8,
    used_quota      int8,
    remaining_quota int8,
    extra           TEXT,
    expires_in      int
);

create table user
(
    id       INTEGER primary key autoincrement,
    username TEXT,
    password TEXT
);

create table file_meta
(
    id          INTEGER
        primary key autoincrement,
    name        TEXT          not null,
    parent_id   INT           not null,
    file_type   int           not null,
    mode        int default 0 not null,
    gid         int default 0 not null,
    uid         int default 0 not null,
    file_length int8          not null,
    status      int           not null,
    deleted     int           not null,
    create_time TIMESTAMP     not null,
    update_time TIMESTAMP     not null
);
create table file_block_meta
(
    id               INTEGER
        primary key autoincrement,
    block_index      int       not null,
    file_part_id     text      not null,
    update_time      TIMESTAMP not null,
    file_modify_time int8      not null,
    deleted          int       not null,
    file_meta_id     int8      not null,
    part_hash        TEXT,
    status           int       not null
);


create table cloud_file_block
(
    id              INTEGER   primary key autoincrement,
    file_block_id   int       not null,
    cloud_meta_id   int       not null,
    cloud_file_id   TEXT,
    cloud_file_hash TEXT,
    status          int       not null,
    deleted         int       not null,
    create_time     TIMESTAMP not null,
    update_time     TIMESTAMP not null
);

create index cloud_file_block_cloud_meta_id_index
    on cloud_file_block (cloud_meta_id);

create table event_message
(
    id           INTEGER primary key autoincrement,
    event_type   int  not null,
    event_result text not null,
    message      int8 not null,
    create_time  int8 not null
);

create table config
(
    id       INTEGER primary key autoincrement,
    property TEXT,
    value    TEXT
);

create table temp_cloud_file
(
    id            integer not null constraint temp_cloud_file_pk primary key autoincrement,
    cloud_meta_id integer,
    cloud_file_id text,
    create_time   timestamp,
    update_time   timestamp,
    status        integer
);

create index temp_cloud_file_status_index on temp_cloud_file (status);

insert into config(property, value) values ('CLOUD_FILE_ROOT',lower( hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || '4' || substr(hex( randomblob(2)), 2) || '-' || substr('AB89', 1 + (abs(random()) % 4) , 1) || substr(hex(randomblob(2)), 2) || '-' ||hex(randomblob(6))));
insert into user(username, password) values ('admin','$2b$12$v/VUPTls0Hib7Q0IZ0LkV.w2Nwlwb5DiO60iUgdLtrfxe7QJrLRF.');
insert into file_meta (name, parent_id, file_type, file_length, status, deleted, create_time, update_time) values ('/',0,2,0,0,0, datetime() ,datetime());
