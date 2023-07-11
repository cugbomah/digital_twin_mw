CREATE TABLE IF NOT EXISTS core_role
(
    id uuid NOT NULL,
    name character varying UNIQUE COLLATE pg_catalog."default" NOT NULL,
    description character varying COLLATE pg_catalog."default",
    "systemRole" boolean NOT NULL DEFAULT false,
    status boolean NOT NULL DEFAULT true,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_role_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS core_user
(
    id uuid NOT NULL,
    "firstName" character varying COLLATE pg_catalog."default" NOT NULL,
    "lastName" character varying COLLATE pg_catalog."default" NOT NULL,
    email character varying UNIQUE COLLATE pg_catalog."default" NOT NULL,
    password character varying COLLATE pg_catalog."default" NOT NULL,
    "roleId" uuid NOT NULL,
    status boolean NOT NULL DEFAULT true,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid,
    "updatedAt" timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_user_pkey PRIMARY KEY (id),
    CONSTRAINT "core_user_roleId_fkey" FOREIGN KEY ("roleId")
        REFERENCES core_role (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_model_type (
  id          SERIAL PRIMARY KEY,
  name    VARCHAR(64) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS core_model
(
    id uuid NOT NULL,
    name character varying UNIQUE COLLATE pg_catalog."default" NOT NULL,
    description character varying COLLATE pg_catalog."default" NOT NULL,
    "typeId" integer NOT NULL,
    picture character varying COLLATE pg_catalog."default",
    "isPublished" boolean NOT NULL DEFAULT false,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_model_pkey PRIMARY KEY (id),
    CONSTRAINT "core_model_typeId_fkey" FOREIGN KEY ("typeId")
        REFERENCES core_model_type (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_model_component (
  id uuid NOT NULL,
  name    VARCHAR(64) NOT NULL UNIQUE,
  "componentAlias"    VARCHAR(64),
  "imageSource"    VARCHAR(64) NOT NULL UNIQUE,
  "containerPort" integer,
  "isExposed" boolean NOT NULL DEFAULT false,
  "modelId" uuid NOT NULL,
  "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
  CONSTRAINT core_model_component_pkey PRIMARY KEY (id),
  CONSTRAINT "core_model_comp_fkey" FOREIGN KEY ("modelId")
        REFERENCES core_model (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);


CREATE TABLE IF NOT EXISTS core_policy
(
    id uuid NOT NULL,
    name character varying COLLATE pg_catalog."default" NOT NULL,
    description character varying COLLATE pg_catalog."default" NOT NULL,
    "modelId" uuid NOT NULL,
    "policyVersion" integer NOT NULL DEFAULT 1,
    "blockAfter" integer NOT NULL DEFAULT 0,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid NOT NULL,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_policy_pkey PRIMARY KEY (id),
    CONSTRAINT "core_twin_modelId_fkey" FOREIGN KEY ("modelId")
        REFERENCES core_model (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_policy_userId_fkey" FOREIGN KEY ("createdBy")
        REFERENCES core_user (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_action_reset_frequency
(
    id          SERIAL PRIMARY KEY,
  name    VARCHAR(64) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS core_policy_action
(
    id uuid NOT NULL,
    "policyId" uuid NOT NULL,
    "endPoint" character varying COLLATE pg_catalog."default" NOT NULL,
    description character varying COLLATE pg_catalog."default" NOT NULL,
    "endPointVerb" character varying COLLATE pg_catalog."default" NOT NULL,
    "actionCount" integer NOT NULL DEFAULT 0,
    "resetFrequencyId" integer NOT NULL DEFAULT 1,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid NOT NULL,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_policy_action_pkey PRIMARY KEY (id),
    CONSTRAINT "core_userId_fkey" FOREIGN KEY ("createdBy")
        REFERENCES core_user (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_action_policyId_fkey" FOREIGN KEY ("policyId")
        REFERENCES core_policy (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_policy_resetFreqId_fkey" FOREIGN KEY ("resetFrequencyId")
        REFERENCES core_action_reset_frequency (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_twin_status
(
    id          SERIAL PRIMARY KEY,
  name    VARCHAR(64) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS core_twin
(
    id uuid NOT NULL,
    name character varying COLLATE pg_catalog."default" NOT NULL,
    "modelId" uuid NOT NULL,
    "policyId" uuid,
    "typeId" integer NOT NULL,
    "twinStatusId" integer NOT NULL DEFAULT 1,
    "networkName" character varying COLLATE pg_catalog."default",
    "twinPort" integer,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid NOT NULL,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
    CONSTRAINT core_twin_pkey PRIMARY KEY (id),
    CONSTRAINT "core_twin_modelId_fkey" FOREIGN KEY ("modelId")
        REFERENCES core_model (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_twin_policyId_fkey" FOREIGN KEY ("policyId")
        REFERENCES core_policy (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,        
    CONSTRAINT "core_twin_createdby_fkey" FOREIGN KEY ("createdBy")
        REFERENCES core_user (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_twin_status_fkey" FOREIGN KEY ("twinStatusId")
        REFERENCES core_twin_status (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_twin_typeId_fkey" FOREIGN KEY ("typeId")
    REFERENCES core_model_type (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_twin_component (
  id uuid NOT NULL,
  name    VARCHAR(64) NOT NULL,
  "componentAlias"    VARCHAR(64),
  "containerPort" integer,
  "containerName" character varying COLLATE pg_catalog."default",
  "hostPort" integer,
  "isExposed" boolean NOT NULL DEFAULT false,
  "twinId" uuid NOT NULL,
  "imageSource" character varying(64) COLLATE pg_catalog."default" NOT NULL,
  "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdBy" uuid,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedBy" uuid,
    "deletedAt" timestamp(6) with time zone,
    "deletedBy" uuid,
  CONSTRAINT core_twin_component_pkey PRIMARY KEY (id),
  CONSTRAINT "core_twin_comp_fkey" FOREIGN KEY ("twinId")
        REFERENCES core_twin (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_policy_violation
(
    id uuid NOT NULL,
    "userId" uuid NOT NULL,
    "modelId" uuid NOT NULL,
    "policyId" uuid NOT NULL,
    "actionId" uuid NOT NULL,
    "twinId" uuid NOT NULL,
    "violatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT core_policy_violation_pkey PRIMARY KEY (id),
    CONSTRAINT "core_violation_userId_fkey" FOREIGN KEY ("userId")
        REFERENCES core_user (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_violation_modelId_fkey" FOREIGN KEY ("modelId")
        REFERENCES core_model (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_violation_policyId_fkey" FOREIGN KEY ("policyId")
        REFERENCES core_policy (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_violation_actionId_fkey" FOREIGN KEY ("actionId")
        REFERENCES core_policy_action (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_violation_twinId_fkey" FOREIGN KEY ("twinId")
        REFERENCES core_twin (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS core_user_subscription
(
    id uuid NOT NULL,
    "userId" uuid NOT NULL,
    "modelId" uuid NOT NULL,
    "isActive" boolean NOT NULL DEFAULT true,
    "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT core_user_sub_pkey PRIMARY KEY (id),
    CONSTRAINT "core_user_sub_userId_fkey" FOREIGN KEY ("userId")
        REFERENCES core_user (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT "core_user_sub_modelId_fkey" FOREIGN KEY ("modelId")
        REFERENCES core_model (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);


-- CREATE TABLE IF NOT EXISTS core_user_model_policy
-- (
--     id uuid NOT NULL,
--     "userId" uuid NOT NULL,
--     "modelId" uuid NOT NULL,
--     "policyId" uuid NOT NULL,
--     "actionId" uuid NOT NULL,
--     "usageCount" integer NOT NULL DEFAULT 0,
--     "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     CONSTRAINT core_ump_pkey PRIMARY KEY (id),
--     CONSTRAINT "core_ump_userId_fkey" FOREIGN KEY ("userId")
--         REFERENCES core_user (id) MATCH SIMPLE
--         ON UPDATE NO ACTION
--         ON DELETE NO ACTION,
--     CONSTRAINT "core_ump_modelId_fkey" FOREIGN KEY ("modelId")
--         REFERENCES core_model (id) MATCH SIMPLE
--         ON UPDATE NO ACTION
--         ON DELETE NO ACTION,
--     CONSTRAINT "core_ump_policyId_fkey" FOREIGN KEY ("policyId")
--         REFERENCES core_policy (id) MATCH SIMPLE
--         ON UPDATE NO ACTION
--         ON DELETE NO ACTION,
--     CONSTRAINT "core_ump_actionId_fkey" FOREIGN KEY ("actionId")
--         REFERENCES core_policy_action (id) MATCH SIMPLE
--         ON UPDATE NO ACTION
--         ON DELETE NO ACTION
-- );

-- CREATE TABLE IF NOT EXISTS core_user_twin
-- (
--     id uuid NOT NULL,
--     "userId" uuid NOT NULL,
--     "twinId" uuid NOT NULL,
--           name character varying COLLATE pg_catalog."default" NOT NULL UNIQUE,
--            port integer NOT NULL UNIQUE DEFAULT 8000,
--     "createdAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     "createdBy" uuid,
--     "updatedAt" timestamp(6) with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     "updatedBy" uuid,
--     "deletedAt" timestamp(6) with time zone,
--     "deletedBy" uuid,
--     CONSTRAINT core_user_twin_pkey PRIMARY KEY (id),
--     CONSTRAINT "core_user_userId_fkey" FOREIGN KEY ("userId")
--         REFERENCES core_user (id) MATCH SIMPLE
--         ON UPDATE SET NULL
--         ON DELETE SET NULL,
--     CONSTRAINT "core_twin_twinId_fkey" FOREIGN KEY ("twinId")
--         REFERENCES core_twin (id) MATCH SIMPLE
--         ON UPDATE NO ACTION
--         ON DELETE NO ACTION
-- );

--Insert Roles--
INSERT INTO core_role (id, name, description, "systemRole")
VALUES ('6b6dab5b-e095-41d7-ab9f-405ec101fbc1', 'Admin', 'System Admin Role', true);
INSERT INTO core_role (id, name, description, "systemRole")
VALUES ('ce3b0789-f2e8-4f42-8e05-b1c3579c414b', 'Owner', 'Twin Owner/Developer Role', false);
INSERT INTO core_role (id, name, description, "systemRole")
VALUES ('053f5c77-f84d-4647-9f74-17e39a9f1a34', 'User', 'Subscriber/User Role', false);

--Insert Model Types--
INSERT INTO core_model_type (name) VALUES ('Docker');
INSERT INTO core_model_type (name) VALUES ('WASM');

--Insert Twin Status--
INSERT INTO core_twin_status (name) VALUES ('Stopped');
INSERT INTO core_twin_status (name) VALUES ('Started');
INSERT INTO core_twin_status (name) VALUES ('Deactivated');
INSERT INTO core_twin_status (name) VALUES ('Deleted');

--Insert Action Reset Frequency--
INSERT INTO core_action_reset_frequency (name) VALUES ('Daily');
INSERT INTO core_action_reset_frequency (name) VALUES ('Weekly');
INSERT INTO core_action_reset_frequency (name) VALUES ('Monthly');
INSERT INTO core_action_reset_frequency (name) VALUES ('Yearly');
INSERT INTO core_action_reset_frequency (name) VALUES ('Never');