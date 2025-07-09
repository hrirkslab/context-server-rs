use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rusqlite::Connection;
use crate::models::flutter::{FlutterComponent, ComponentType, ArchitectureLayer, RiverpodScope, WidgetType};
use crate::repositories::FlutterRepository;
use rmcp::model::ErrorData as McpError;

/// SQLite implementation of FlutterRepository
pub struct SqliteFlutterRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteFlutterRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    fn parse_component_type(component_type_str: &str) -> ComponentType {
        match component_type_str {
            "widget" => ComponentType::Widget,
            "provider" => ComponentType::Provider,
            "service" => ComponentType::Service,
            "repository" => ComponentType::Repository,
            "model" => ComponentType::Model,
            "utility" => ComponentType::Utility,
            _ => ComponentType::Widget,
        }
    }

    fn parse_architecture_layer(architecture_layer_str: &str) -> ArchitectureLayer {
        match architecture_layer_str {
            "presentation" => ArchitectureLayer::Presentation,
            "domain" => ArchitectureLayer::Domain,
            "data" => ArchitectureLayer::Data,
            "core" => ArchitectureLayer::Core,
            _ => ArchitectureLayer::Presentation,
        }
    }

    fn parse_riverpod_scope(scope_str: Option<&str>) -> Option<RiverpodScope> {
        scope_str.and_then(|s| match s {
            "global" => Some(RiverpodScope::Global),
            "scoped" => Some(RiverpodScope::Scoped),
            "local" => Some(RiverpodScope::Local),
            _ => None,
        })
    }

    fn parse_widget_type(widget_type_str: Option<&str>) -> Option<WidgetType> {
        widget_type_str.and_then(|s| match s {
            "stateless_widget" => Some(WidgetType::StatelessWidget),
            "stateful_widget" => Some(WidgetType::StatefulWidget),
            "consumer_widget" => Some(WidgetType::ConsumerWidget),
            "consumer_stateful_widget" => Some(WidgetType::ConsumerStatefulWidget),
            "hook_widget" => Some(WidgetType::HookWidget),
            "hook_consumer_widget" => Some(WidgetType::HookConsumerWidget),
            _ => None,
        })
    }

    fn component_type_to_string(component_type: &ComponentType) -> &'static str {
        match component_type {
            ComponentType::Widget => "widget",
            ComponentType::Provider => "provider",
            ComponentType::Service => "service",
            ComponentType::Repository => "repository",
            ComponentType::Model => "model",
            ComponentType::Utility => "utility",
        }
    }

    fn architecture_layer_to_string(architecture_layer: &ArchitectureLayer) -> &'static str {
        match architecture_layer {
            ArchitectureLayer::Presentation => "presentation",
            ArchitectureLayer::Domain => "domain",
            ArchitectureLayer::Data => "data",
            ArchitectureLayer::Core => "core",
        }
    }

    fn riverpod_scope_to_string(scope: &Option<RiverpodScope>) -> Option<&'static str> {
        scope.as_ref().map(|s| match s {
            RiverpodScope::Global => "global",
            RiverpodScope::Scoped => "scoped",
            RiverpodScope::Local => "local",
        })
    }

    fn widget_type_to_string(widget_type: &Option<WidgetType>) -> Option<&'static str> {
        widget_type.as_ref().map(|w| match w {
            WidgetType::StatelessWidget => "stateless_widget",
            WidgetType::StatefulWidget => "stateful_widget",
            WidgetType::ConsumerWidget => "consumer_widget",
            WidgetType::ConsumerStatefulWidget => "consumer_stateful_widget",
            WidgetType::HookWidget => "hook_widget",
            WidgetType::HookConsumerWidget => "hook_consumer_widget",
        })
    }
}

#[async_trait]
impl FlutterRepository for SqliteFlutterRepository {
    async fn create(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError> {
        let db = self.db.lock().unwrap();
        
        let dependencies_json = serde_json::to_string(&component.dependencies)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        
        db.execute(
            "INSERT INTO flutter_components (id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, riverpod_scope, widget_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &component.id,
                &component.project_id,
                &component.component_name,
                Self::component_type_to_string(&component.component_type),
                Self::architecture_layer_to_string(&component.architecture_layer),
                component.file_path.as_deref(),
                &dependencies_json,
                Self::riverpod_scope_to_string(&component.riverpod_scope),
                Self::widget_type_to_string(&component.widget_type),
                component.created_at.as_deref(),
                component.updated_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(component.clone())
    }

    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<FlutterComponent>, McpError> {
        let db = self.db.lock().unwrap();
        let mut components = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, riverpod_scope, widget_type, created_at, updated_at FROM flutter_components WHERE project_id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let component_rows = stmt.query_map([project_id], |row| {
            let component_type_str: String = row.get(3)?;
            let architecture_layer_str: String = row.get(4)?;
            let dependencies_str: String = row.get(6)?;
            let riverpod_scope_str: Option<String> = row.get(7)?;
            let widget_type_str: Option<String> = row.get(8)?;
            
            let component_type = Self::parse_component_type(&component_type_str);
            let architecture_layer = Self::parse_architecture_layer(&architecture_layer_str);
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();
            let riverpod_scope = Self::parse_riverpod_scope(riverpod_scope_str.as_deref());
            let widget_type = Self::parse_widget_type(widget_type_str.as_deref());

            Ok(FlutterComponent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_name: row.get(2)?,
                component_type,
                architecture_layer,
                file_path: row.get(5)?,
                dependencies,
                riverpod_scope,
                widget_type,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for component in component_rows {
            match component {
                Ok(component) => components.push(component),
                Err(e) => tracing::warn!("Failed to parse Flutter component: {}", e),
            }
        }

        Ok(components)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<FlutterComponent>, McpError> {
        let db = self.db.lock().unwrap();
        
        let mut stmt = db.prepare("SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, riverpod_scope, widget_type, created_at, updated_at FROM flutter_components WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let mut component_iter = stmt.query_map([id], |row| {
            let component_type_str: String = row.get(3)?;
            let architecture_layer_str: String = row.get(4)?;
            let dependencies_str: String = row.get(6)?;
            let riverpod_scope_str: Option<String> = row.get(7)?;
            let widget_type_str: Option<String> = row.get(8)?;
            
            let component_type = Self::parse_component_type(&component_type_str);
            let architecture_layer = Self::parse_architecture_layer(&architecture_layer_str);
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();
            let riverpod_scope = Self::parse_riverpod_scope(riverpod_scope_str.as_deref());
            let widget_type = Self::parse_widget_type(widget_type_str.as_deref());

            Ok(FlutterComponent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_name: row.get(2)?,
                component_type,
                architecture_layer,
                file_path: row.get(5)?,
                dependencies,
                riverpod_scope,
                widget_type,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match component_iter.next() {
            Some(Ok(component)) => Ok(Some(component)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn update(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError> {
        let db = self.db.lock().unwrap();
        
        let dependencies_json = serde_json::to_string(&component.dependencies)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        
        db.execute(
            "UPDATE flutter_components SET project_id = ?, component_name = ?, component_type = ?, architecture_layer = ?, file_path = ?, dependencies = ?, riverpod_scope = ?, widget_type = ?, updated_at = ? WHERE id = ?",
            (
                &component.project_id,
                &component.component_name,
                Self::component_type_to_string(&component.component_type),
                Self::architecture_layer_to_string(&component.architecture_layer),
                component.file_path.as_deref(),
                &dependencies_json,
                Self::riverpod_scope_to_string(&component.riverpod_scope),
                Self::widget_type_to_string(&component.widget_type),
                component.updated_at.as_deref(),
                &component.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(component.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();
        
        let rows_affected = db.execute("DELETE FROM flutter_components WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }

    async fn find_by_architecture_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FlutterComponent>, McpError> {
        let db = self.db.lock().unwrap();
        let mut components = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, riverpod_scope, widget_type, created_at, updated_at FROM flutter_components WHERE project_id = ? AND architecture_layer = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let component_rows = stmt.query_map([project_id, layer], |row| {
            let component_type_str: String = row.get(3)?;
            let architecture_layer_str: String = row.get(4)?;
            let dependencies_str: String = row.get(6)?;
            let riverpod_scope_str: Option<String> = row.get(7)?;
            let widget_type_str: Option<String> = row.get(8)?;
            
            let component_type = Self::parse_component_type(&component_type_str);
            let architecture_layer = Self::parse_architecture_layer(&architecture_layer_str);
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();
            let riverpod_scope = Self::parse_riverpod_scope(riverpod_scope_str.as_deref());
            let widget_type = Self::parse_widget_type(widget_type_str.as_deref());

            Ok(FlutterComponent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_name: row.get(2)?,
                component_type,
                architecture_layer,
                file_path: row.get(5)?,
                dependencies,
                riverpod_scope,
                widget_type,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for component in component_rows {
            match component {
                Ok(component) => components.push(component),
                Err(e) => tracing::warn!("Failed to parse Flutter component: {}", e),
            }
        }

        Ok(components)
    }
}
