-- Example Constraints for OpenClaw Autonomous Agent
-- These constraints tell OpenClaw what operations are safe and what guardrails to enforce

-- Resource Limits
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_001', 'project_001', 'resource_limit', 'Max Database Connections', 'Maximum simultaneous connections to production database', 'service:database', 'max_connections:100', 'critical', 1, datetime('now'), datetime('now'), '["production","database"]', 'REJECT_IF_EXCEEDED');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_002', 'project_001', 'resource_limit', 'Max Memory Usage', 'Maximum memory per service container', 'service:api', 'max_memory:2GB', 'high', 1, datetime('now'), datetime('now'), '["production","performance"]', 'KILL_IF_EXCEEDED');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_003', 'project_001', 'resource_limit', 'Max CPU Usage', 'Maximum CPU utilization', 'deployment:kubernetes', 'max_cpu:80%', 'high', 1, datetime('now'), datetime('now'), '["production","performance"]', 'THROTTLE_IF_EXCEEDED');

-- Safety Guards
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_004', 'project_001', 'safety_guard', 'Staging Test Required', 'All changes must be tested in staging first', 'deployment:all', 'require_staging_test:true', 'critical', 1, datetime('now'), datetime('now'), '["deployment","quality"]', 'REQUIRE_APPROVAL');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_005', 'project_001', 'safety_guard', 'Change Window', 'Only deploy during business hours Mon-Fri 9am-5pm', 'deployment:production', 'time_window:mon-fri_09:00-17:00_UTC', 'high', 1, datetime('now'), datetime('now'), '["production","deployment"]', 'DEFER_TO_WINDOW');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_006', 'project_001', 'safety_guard', 'No Direct Database Writes', 'Use migrations only, no direct schema modifications', 'database:schema', 'method:migrations_only', 'critical', 1, datetime('now'), datetime('now'), '["database","production"]', 'BLOCK_OPERATION');

-- Rollback Procedures
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_007', 'project_001', 'rollback_procedure', 'Quick Rollback', 'Have rollback ready before deploying', 'deployment:production', 'procedure:git_revert_or_helm_rollback', 'critical', 1, datetime('now'), datetime('now'), '["deployment","recovery"]', 'REQUIRE_BEFORE_DEPLOY');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_008', 'project_001', 'rollback_procedure', 'Database Backup', 'Backup database before schema changes', 'database:backup', 'procedure:pg_dump_or_cloud_backup', 'critical', 1, datetime('now'), datetime('now'), '["database","recovery"]', 'REQUIRE_BEFORE_CHANGE');

-- Approval Required
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_009', 'project_001', 'approval_required', 'Production Deployment', 'Manual approval required for production changes', 'deployment:production', 'approval_threshold:2_team_leads', 'critical', 1, datetime('now'), datetime('now'), '["production","deployment"]', 'REQUIRE_APPROVAL');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_010', 'project_001', 'approval_required', 'Database Migration', 'Manual approval for large data migrations', 'database:migration', 'approval_threshold:1_senior_dba', 'critical', 1, datetime('now'), datetime('now'), '["database","production"]', 'REQUIRE_APPROVAL');

-- Performance Targets
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_011', 'project_001', 'performance_target', 'API Response Time', 'API endpoints must respond within 200ms', 'service:api', 'p99_latency:200ms', 'high', 1, datetime('now'), datetime('now'), '["performance","sla"]', 'MONITOR_AND_ALERT');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_012', 'project_001', 'performance_target', 'Availability SLA', 'System must maintain 99.9% uptime', 'service:all', 'availability:99.9%', 'critical', 1, datetime('now'), datetime('now'), '["performance","sla"]', 'MONITOR_AND_ALERT');

-- Security Requirements
INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_013', 'project_001', 'security_requirement', 'No Secrets in Logs', 'Never log API keys, passwords, or tokens', 'logging:all', 'pattern:no_credentials_in_logs', 'critical', 1, datetime('now'), datetime('now'), '["security","logging"]', 'BLOCK_IF_DETECTED');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_014', 'project_001', 'security_requirement', 'TLS Only', 'All external communication must use TLS 1.3+', 'network:external', 'protocol:tls_1.3_or_higher', 'critical', 1, datetime('now'), datetime('now'), '["security","network"]', 'ENFORCE');

INSERT INTO constraints (id, project_id, constraint_type, name, description, target, value, severity, enabled, created_at, last_modified_at, tags, enforcement_action) VALUES
('const_015', 'project_001', 'security_requirement', 'Authentication Required', 'All APIs require authentication (OAuth 2.0 or mTLS)', 'api:all', 'auth_method:oauth2_or_mtls', 'critical', 1, datetime('now'), datetime('now'), '["security","api"]', 'ENFORCE');
