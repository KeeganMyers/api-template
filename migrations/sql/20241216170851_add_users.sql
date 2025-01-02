CREATE TYPE target AS ENUM ('user_permission', 'user');

create table users (
  id uuid primary key default uuid_generate_v7() not null,
  external_id text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  last_login timestamptz,
  display_name varchar(100),
  email varchar(256),
);

create table user_permissions (
  id uuid primary key default uuid_generate_v7() not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  user_id uuid NOT NULL,
  target target NOT NULL,
  create_record bool NOT NULL default false,
  update_record bool NOT NULL default false,
  view_record bool NOT NULL default false,
  delete_record bool NOT NULL default false
);

create or replace view user_readmodels_v as
select u.id,external_id,display_name,email,p.permissions
from users u
left join (select json_agg(p.*) as permissions, p.user_id from (select id,target,create_record,update_record,view_record,delete_record,user_id from user_permissions) p group by p.user_id) p
on u.id = p.user_id

create table user_readmodels as select * from user_readmodels_v;
ALTER TABLE user_readmodels ADD CONSTRAINT unique_user_id UNIQUE(id);
