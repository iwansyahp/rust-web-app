---- Base app schema

-- User
CREATE TABLE "user" (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  username varchar(128) NOT NULL UNIQUE,

  -- Auth
  pwd varchar(256),
  pwd_salt uuid NOT NULL DEFAULT gen_random_uuid(),
  token_salt uuid NOT NULL DEFAULT gen_random_uuid()
);

-- Project
CREATE TABLE project (
  -- PK
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  -- Properties
  owner_id BIGINT NOT NULL,
  name varchar(256) NOT NULL
);

-- Task
CREATE TABLE task (
  -- PK
  id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1000) PRIMARY KEY,

  -- FK
  project_id BIGINT NOT NULL,
  
  -- Properties
  title varchar(256) NOT NULL,
  done bool NOT NULL DEFAULT false
);

ALTER TABLE task ADD CONSTRAINT fk_project
  FOREIGN KEY (project_id) REFERENCES project(id)
  ON DELETE CASCADE;