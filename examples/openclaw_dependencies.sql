-- Example Component Dependencies for OpenClaw
-- These help OpenClaw understand the impact of changes

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_001', 'project_001', 'api-service', 'service', 'database', 'service', 'depends_on', 'API queries the database for all requests', 'critical', 'All API requests will fail', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_002', 'project_001', 'api-service', 'service', 'redis-cache', 'service', 'depends_on', 'API uses Redis for session caching', 'high', 'Session handling degrades, increased database load', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_003', 'project_001', 'worker-service', 'service', 'database', 'service', 'depends_on', 'Workers process jobs from database queue', 'critical', 'No background jobs will process', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_004', 'project_001', 'worker-service', 'service', 'message-queue', 'service', 'depends_on', 'Workers consume messages from queue', 'critical', 'Event processing stops', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_005', 'project_001', 'notification-service', 'service', 'message-queue', 'service', 'depends_on', 'Notifications are sent via queue', 'high', 'Users will not receive notifications', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_006', 'project_001', 'notification-service', 'service', 'email-service', 'service', 'depends_on', 'Email notifications are sent via third-party service', 'high', 'Email notifications fail silently', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_007', 'project_001', 'load-balancer', 'infrastructure', 'api-service', 'service', 'routes_to', 'Load balancer distributes traffic to API instances', 'critical', 'API is not accessible', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_008', 'project_001', 'database', 'service', 'storage-backend', 'infrastructure', 'depends_on', 'Database persists to storage backend', 'critical', 'Data loss, database cannot start', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_009', 'project_001', 'redis-cache', 'service', 'memory-store', 'infrastructure', 'depends_on', 'Redis uses memory store for caching', 'high', 'Caching disabled, performance degrades', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_010', 'project_001', 'kubernetes-cluster', 'infrastructure', 'database', 'service', 'runs', 'Database pods run on Kubernetes cluster', 'critical', 'Database becomes unavailable', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_011', 'project_001', 'monitoring-stack', 'infrastructure', 'prometheus', 'service', 'depends_on', 'Monitoring collects metrics from Prometheus', 'medium', 'Metrics not collected, blind spot in monitoring', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_012', 'project_001', 'api-service', 'service', 'auth-service', 'service', 'depends_on', 'API validates requests with auth service', 'critical', 'All requests fail authentication', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_013', 'project_001', 'auth-service', 'service', 'ldap-directory', 'service', 'depends_on', 'Auth service validates credentials via LDAP', 'critical', 'User authentication fails', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_014', 'project_001', 'backup-service', 'service', 'database', 'service', 'depends_on', 'Backup service reads from database', 'high', 'No backups will be created, risk of data loss', datetime('now'));

INSERT INTO component_dependencies (id, project_id, source_component, source_type, target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at) VALUES
('dep_015', 'project_001', 'migration-service', 'service', 'database', 'service', 'depends_on', 'Schema migrations are applied to database', 'critical', 'Database schema cannot be updated', datetime('now'));
