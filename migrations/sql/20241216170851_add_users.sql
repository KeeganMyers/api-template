CREATE TYPE role AS ENUM ('admin', 'user');

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v7() NOT NULL,
  external_id TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_login TIMESTAMPTZ,
  display_name VARCHAR(100),
  email VARCHAR(256),
  role role NOT NULL
);
