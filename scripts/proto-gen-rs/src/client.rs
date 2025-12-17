use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
use prost_types::{FileDescriptorSet, ServiceDescriptorProto};
use std::{fs, path::Path};

/// Generates wrapper clients for *every* service found
pub(crate) fn generate_client<P: AsRef<Path>>(
    src_dir: &P,
    proto_dir: &P,
    fds: &FileDescriptorSet,
) -> Result<()> {
    let file = find_target_file(&fds);

    // Panic if the file contains multiple services
    if file.service.len() != 1 {
        panic!(
            "Proto file '{}' contains {} services, but exactly 1 is required.",
            file.name.as_ref().unwrap_or(&String::new()),
            file.service.len()
        );
    }

    // Panic if the file contains multiple services
    if file.service.len() != 1 {
        panic!(
            "Proto file '{}' contains {} services, but exactly 1 is required.",
            file.name.as_ref().unwrap_or(&String::new()),
            file.service.len()
        );
    }

    // Get the service name from the path
    let service_name = std::path::Path::new(proto_dir.as_ref())
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Generate the client.rs
    let service = &file.service[0];
    let code = generate_client_code(service, &service_name)?;
    let fname = format!("{}/client.rs", src_dir.as_ref().to_string_lossy());
    fs::write(fname, code)?;

    Ok(())
}

/// Generate a client module for a single service
fn generate_client_code(svc: &ServiceDescriptorProto, svc_name: &str) -> Result<String> {
    let svc_name = svc_name.to_upper_camel_case();

    let (trait_methods, impl_methods, mock_field_decls, mock_field_inits, mock_impl) =
        generate_methods(svc)?;

    let imports = generate_imports(svc);

    Ok(format!(
        r#"// This file is generated.
use crate::GRPC_PORT;
use crate::SERVICE_NAME;
{imports}
use setup::{{middleware::tracing::TracingServiceClient, patched_host}};
use std::{{error::Error, str::FromStr as _}};
use tonic::transport::{{Channel, Endpoint}};
use tonic::{{Request, Response, Status, async_trait}};

#[derive(Clone)]
pub struct {svc_name}Client(ApiServiceClient<TracingServiceClient<Channel>>);

impl {svc_name}Client {{
    pub async fn new() -> Result<Self, Box<dyn Error>> {{
        let host = patched_host(String::from(SERVICE_NAME));
        let endpoint = Endpoint::from_str(&format!("http://{{host}}:{{GRPC_PORT}}"))?;
        let channel = endpoint.connect().await?;
        let client = TracingServiceClient::new(channel);
        let client = ApiServiceClient::new(client);

        Ok(Self(client))
    }}
}}

#[rustfmt::skip]
#[async_trait]
pub trait I{svc_name}Client: Send + Sync + 'static {{
{trait_methods}
}}

#[rustfmt::skip]
#[async_trait]
impl I{svc_name}Client for {svc_name}Client {{
{impl_methods}
}}

#[cfg(feature = "testutils")]
pub mod testutils {{
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{{Request, Response, Status}};

    #[rustfmt::skip]
    pub struct Mock{svc_name}Client {{
{mock_field_decls}
    }}

    impl Default for Mock{svc_name}Client {{
        fn default() -> Self {{
            Self {{
{mock_field_inits}
            }}
        }}
    }}

    #[rustfmt::skip]
    #[async_trait]
    impl I{svc_name}Client for Mock{svc_name}Client {{
{mock_impl}
    }}
}}
"#,
        imports = imports,
        svc_name = svc_name,
        trait_methods = trait_methods,
        impl_methods = impl_methods,
        mock_field_decls = mock_field_decls,
        mock_field_inits = mock_field_inits,
        mock_impl = mock_impl,
    ))
}

/// Generates all RPC method blocks, plus separate mock decls and inits
fn generate_methods(
    svc: &ServiceDescriptorProto,
) -> Result<(String, String, String, String, String)> {
    let mut trait_methods_vec = Vec::new();
    let mut impl_methods_vec = Vec::new();
    let mut mock_field_decls_vec = Vec::new();
    let mut mock_field_inits_vec = Vec::new();
    let mut mock_impl_vec = Vec::new();

    for m in &svc.method {
        let method_name = m.name.as_ref().unwrap();
        let method_snake = method_name.to_snake_case();

        let input = rust_type(m.input_type());
        let output = rust_type(m.output_type());

        // trait signature
        trait_methods_vec.push(format!(
        "    async fn {method_snake}(&self, req: Request<{input}>) -> Result<Response<{output}>, Status>;",
        method_snake = method_snake,
        input = input,
        output = output
    ));

        // impl calling tonic client
        impl_methods_vec.push(format!(
            r#"    async fn {method_snake}(&self, req: Request<{input}>) -> Result<Response<{output}>, Status> {{
        self.0.clone().{method_snake}(req).await
    }}"#,
            method_snake = method_snake,
            input = input,
            output = output
        ));

        // mock struct field declarations
        mock_field_decls_vec.push(format!(
        "        pub {method_snake}_req: Mutex<Option<{input}>>,\n        pub {method_snake}_resp: Mutex<Option<Result<{output}, Status>>>,",
        method_snake = method_snake,
        input = input,
        output = output
    ));

        // mock initializers
        mock_field_inits_vec.push(format!(
        "                {method_snake}_req: Mutex::new(None),\n                {method_snake}_resp: Mutex::new(None),",
        method_snake = method_snake
    ));

        // mock method impl
        mock_impl_vec.push(format!(
            r#"        async fn {method_snake}(&self, req: Request<{input}>) -> Result<Response<{output}>, Status> {{
            *self.{method_snake}_req.lock().await = Some(req.into_inner());
            self.{method_snake}_resp.lock().await.take().unwrap().map(Response::new)
        }}"#,
            method_snake = method_snake,
            input = input,
            output = output
        ));
    }

    let trait_methods = trait_methods_vec.join("\n");
    let impl_methods = impl_methods_vec.join("\n");
    let mock_field_decls = mock_field_decls_vec.join("\n");
    let mock_field_inits = mock_field_inits_vec.join("\n");
    let mock_impl = mock_impl_vec.join("\n");

    Ok((
        trait_methods,
        impl_methods,
        mock_field_decls,
        mock_field_inits,
        mock_impl,
    ))
}

/// Extract "MyMessage" from ".mypackage.MyMessage" (or "MyMessage")
fn rust_type(proto_type: &str) -> String {
    proto_type
        .trim_start_matches('.')
        .split('.')
        .next_back()
        .unwrap()
        .to_string()
}

/// Find all messages used by the service
fn generate_imports(svc: &prost_types::ServiceDescriptorProto) -> String {
    use std::collections::BTreeSet;
    let mut imports: BTreeSet<String> = BTreeSet::new();

    for m in &svc.method {
        imports.insert(rust_type(m.input_type()));
        imports.insert(rust_type(m.output_type()));
    }

    imports.insert("api_service_client::ApiServiceClient".to_string());

    imports
        .into_iter()
        .map(|ty| format!("use crate::proto::{};", ty))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the `api.proto` file in the descriptor
fn find_target_file<'a>(fds: &'a FileDescriptorSet) -> &'a prost_types::FileDescriptorProto {
    let candidates: Vec<_> = fds
        .file
        .iter()
        .filter(|f| {
            f.name
                .as_ref()
                .map(|n| n.ends_with("api.proto"))
                .unwrap_or(false)
        })
        .collect();

    if candidates.len() != 1 {
        panic!(
            "Expected exactly 1 api.proto file in descriptor, found {}",
            candidates.len()
        );
    }

    candidates[0]
}
