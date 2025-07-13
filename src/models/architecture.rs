use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchitectureLayer {
    Presentation,
    Domain,
    Data,
    Core,
}

impl FromStr for ArchitectureLayer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "presentation" => Ok(ArchitectureLayer::Presentation),
            "domain" => Ok(ArchitectureLayer::Domain),
            "data" => Ok(ArchitectureLayer::Data),
            "core" => Ok(ArchitectureLayer::Core),
            _ => Err(format!("Unknown architecture layer: {s}")),
        }
    }
}

impl fmt::Display for ArchitectureLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchitectureLayer::Presentation => write!(f, "presentation"),
            ArchitectureLayer::Domain => write!(f, "domain"),
            ArchitectureLayer::Data => write!(f, "data"),
            ArchitectureLayer::Core => write!(f, "core"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Widget,
    Provider,
    Service,
    Repository,
    Model,
    Utility,
    Controller,
    View,
}

impl FromStr for ComponentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "widget" => Ok(ComponentType::Widget),
            "provider" => Ok(ComponentType::Provider),
            "service" => Ok(ComponentType::Service),
            "repository" => Ok(ComponentType::Repository),
            "model" => Ok(ComponentType::Model),
            "utility" => Ok(ComponentType::Utility),
            "controller" => Ok(ComponentType::Controller),
            "view" => Ok(ComponentType::View),
            _ => Err(format!("Unknown component type: {s}")),
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::Widget => write!(f, "widget"),
            ComponentType::Provider => write!(f, "provider"),
            ComponentType::Service => write!(f, "service"),
            ComponentType::Repository => write!(f, "repository"),
            ComponentType::Model => write!(f, "model"),
            ComponentType::Utility => write!(f, "utility"),
            ComponentType::Controller => write!(f, "controller"),
            ComponentType::View => write!(f, "view"),
        }
    }
}
