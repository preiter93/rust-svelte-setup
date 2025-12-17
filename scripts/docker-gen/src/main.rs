//! Generates optimized Dockerfiles for Rust workspace services.
//! Analyzes dependencies and creates minimal workspace configs.

use minijinja::{Environment, context};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use toml::Value;

#[derive(Debug, Serialize, Deserialize)]
struct CopyFile {
    src: String,
    dest: String,
}

fn get_workspace_dependencies(
    service_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let workspace_members = parse_workspace_members()?;
    let mut required_members = HashSet::new();
    let mut to_process = vec![service_name.to_string()];

    // Process dependencies transitively
    while let Some(current) = to_process.pop() {
        if required_members.contains(&current) {
            continue;
        }

        required_members.insert(current.clone());

        let member_deps = if current == service_name {
            parse_service_dependencies_for_path("Cargo.toml")?
        } else {
            parse_service_dependencies_for_path(&format!("../{}/Cargo.toml", current))?
        };

        for dep in member_deps {
            if workspace_members.contains(&dep) && !required_members.contains(&dep) {
                to_process.push(dep);
            }
        }
    }

    let mut sorted_members: Vec<String> = required_members.into_iter().collect();
    sorted_members.sort();
    Ok(sorted_members)
}

fn parse_workspace_members() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let workspace_content = fs::read_to_string("../Cargo.toml")?;
    let workspace_toml: Value = toml::from_str(&workspace_content)?;

    let members = workspace_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(members)
}

fn parse_service_dependencies_for_path(
    cargo_path: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let service_content = fs::read_to_string(cargo_path)?;
    let service_toml: Value = toml::from_str(&service_content)?;
    let workspace_members = parse_workspace_members()?;
    let mut dependencies = Vec::new();

    let extract_path_deps = |deps: &toml::map::Map<String, Value>| {
        let mut path_deps = Vec::new();
        for (dep_name, dep_value) in deps {
            // Check if it's a workspace member
            if workspace_members.contains(dep_name) {
                path_deps.push(dep_name.clone());
                continue;
            }

            // Check if dependency is a workspace member
            if let Some(dep_table) = dep_value.as_table() {
                if let Some(path) = dep_table.get("path").and_then(|p| p.as_str()) {
                    if let Some(member_path) = path.strip_prefix("../") {
                        if workspace_members.contains(&member_path.to_string()) {
                            path_deps.push(member_path.to_string());
                        }
                    }
                }
            }
        }

        path_deps
    };

    // Check dependencies
    if let Some(deps) = service_toml.get("dependencies").and_then(|d| d.as_table()) {
        dependencies.extend(extract_path_deps(deps));
    }

    // Check dev-dependencies
    if let Some(dev_deps) = service_toml
        .get("dev-dependencies")
        .and_then(|d| d.as_table())
    {
        dependencies.extend(extract_path_deps(dev_deps));
    }

    Ok(dependencies)
}

fn create_minimal_workspace(
    service_name: &str,
    required_members: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_content = fs::read_to_string("../Cargo.toml")?;
    let mut workspace_toml: toml::Value = toml::from_str(&workspace_content)?;

    if let Some(workspace) = workspace_toml.get_mut("workspace") {
        if let Some(workspace_table) = workspace.as_table_mut() {
            workspace_table.insert(
                "members".to_string(),
                toml::Value::Array(
                    required_members
                        .iter()
                        .map(|m| toml::Value::String(m.clone()))
                        .collect(),
                ),
            );
        }
    }

    let minimal_workspace_toml = toml::to_string_pretty(&workspace_toml)?;

    fs::create_dir_all("../.docker-gen")?;
    fs::write(
        &format!("../.docker-gen/Cargo.toml.{}", service_name),
        minimal_workspace_toml,
    )?;

    Ok(())
}

fn generate_dockerfile(
    service_name: &str,
    required_members: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    let template_content = include_str!("../templates/Dockerfile.j2");
    env.add_template("dockerfile", template_content)?;

    let copy_files = build_copy_files(service_name, required_members);

    let template = env.get_template("dockerfile")?;
    let rendered = template.render(context! {
        service_name => service_name,
        copy_files => copy_files
    })?;

    fs::write("Dockerfile", rendered)?;
    Ok(())
}

fn build_copy_files(service_name: &str, required_members: &[String]) -> Vec<CopyFile> {
    let mut copy_files = vec![
        CopyFile {
            src: format!("../.docker-gen/Cargo.toml.{}", service_name),
            dest: "Cargo.toml".to_string(),
        },
        CopyFile {
            src: "../Cargo.lock".to_string(),
            dest: "Cargo.lock".to_string(),
        },
    ];

    for member in required_members {
        copy_files.push(CopyFile {
            src: format!("../{}", member),
            dest: member.clone(),
        });
    }

    copy_files
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let service_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Could not determine service name from current directory")?;

    // Resolve member dependencies
    let required_members = get_workspace_dependencies(service_name)?;
    println!("Workspace dependencies: {:?}", required_members);

    // Create minimal workspace config
    create_minimal_workspace(service_name, &required_members)?;

    // Generate Dockerfile
    generate_dockerfile(service_name, &required_members)?;

    Ok(())
}
