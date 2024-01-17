use std::{
    io::{BufRead, BufReader},
    net::Ipv4Addr,
    path::PathBuf,
    str::FromStr,
};

use cidr::Ipv4Cidr;
use clap::Parser;
use log::info;
use poem::{
    http::StatusCode,
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Response, Route,
};
use poem_openapi::{
    param::Path,
    payload::Json,
    registry::{MetaMediaType, MetaResponse, MetaResponses, Registry},
    types::{ToJSON, Type},
    ApiResponse, Object, OpenApi, OpenApiService,
};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env, default_value = "3000", help = "Port to listen on")]
    port: u16,
    #[clap(long, env, help = "Path to the country-ip-blocks repository")]
    repo_path: PathBuf,
}

#[derive(Debug)]
struct Api {
    cidr_blocks: Vec<CidrBlock>,
}

#[derive(Debug)]
struct CidrBlock {
    cidr: Ipv4Cidr,
    country: String,
}

#[derive(Debug)]
pub enum ApiError {
    InvalidIpAddress,
    IpAddressNotFound,
}

#[derive(Debug, Object)]
struct ApiErrorResponse {
    code: u32,
    message: String,
}

#[derive(Debug, Clone, Object)]
struct IpGeolocation {
    country: String,
}

impl Api {
    fn from_repo(repo_path: &std::path::Path) -> anyhow::Result<Self> {
        let mut cidr_blocks = vec![];

        for entry in std::fs::read_dir(repo_path.join("ipv4"))? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().is_some_and(|value| value == "cidr") {
                let country_code = file_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("unable to read file name"))?
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid file name"))?
                    .split_once('.')
                    .expect("already checked that extension exists")
                    .0
                    .to_uppercase();

                if country_code.len() != 2 {
                    anyhow::bail!("invalid country code: {}", country_code);
                }

                let mut file = std::fs::File::open(&file_path)?;
                let reader = BufReader::new(&mut file);
                for line in reader.lines() {
                    let line = line?;

                    let cidr: Ipv4Cidr = line.parse()?;

                    cidr_blocks.push(CidrBlock {
                        cidr,
                        country: country_code.clone(),
                    })
                }
            }
        }

        Ok(Self { cidr_blocks })
    }

    // This implementation is extremely inefficient, with O(n) for each lookup. This can be
    // optimized with a sorted list of CIDR blocks, and use binary search to reduce the steps to
    // O(log n). Though slow and inefficient, it's good enough for an MVP.
    //
    // TODO: optimize with sorted CIDR blocks and binary search.
    fn get_ipv4_country(&self, ip_address: &Ipv4Addr) -> Option<String> {
        for block in self.cidr_blocks.iter() {
            if block.cidr.contains(ip_address) {
                return Some(block.country.clone());
            }
        }

        None
    }
}

#[OpenApi]
impl Api {
    #[oai(path = "/ipv4/:ip_address", method = "get")]
    /// Gets the two-letter ISO 3166 country code associated with the IPv4 address
    async fn get_ipv4(&self, ip_address: Path<String>) -> Result<Json<IpGeolocation>, ApiError> {
        let ip_address =
            Ipv4Addr::from_str(&ip_address.0).map_err(|_| ApiError::InvalidIpAddress)?;

        match self.get_ipv4_country(&ip_address) {
            Some(country) => Ok(Json(IpGeolocation { country })),
            None => Err(ApiError::IpAddressNotFound),
        }
    }
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidIpAddress => StatusCode::BAD_REQUEST,
            Self::IpAddressNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl ApiResponse for ApiError {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(404),
                content: vec![MetaMediaType {
                    content_type: "application/json",
                    schema: ApiErrorResponse::schema_ref(),
                }],
                headers: vec![],
            }],
        }
    }

    fn register(registry: &mut Registry) {
        <ApiErrorResponse as Type>::register(registry);
    }
}

impl From<ApiError> for poem::Error {
    fn from(value: ApiError) -> Self {
        let status_code = value.status_code();
        let response: ApiErrorResponse = value.into();

        Self::from_response(
            Response::builder()
                .status(status_code)
                .content_type("application/json")
                .body(response.to_json_string()),
        )
    }
}

impl From<ApiError> for ApiErrorResponse {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::InvalidIpAddress => Self {
                code: 100,
                message: "Invalid IP address".into(),
            },
            ApiError::IpAddressNotFound => Self {
                code: 101,
                message: "IP address not covered in database".into(),
            },
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "ipgeo=debug,poem=debug");
    }
    env_logger::init();

    let cli = Cli::parse();

    let cors = Cors::new();

    let api = Api::from_repo(&cli.repo_path)?;
    let api_service = OpenApiService::new(api, "ipgeod", env!("CARGO_PKG_VERSION"));

    let app = Route::new()
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/swagger", api_service.swagger_ui())
        .nest("/", api_service.with(cors).with(Tracing));

    let server = poem::Server::new(TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), cli.port)));
    info!("Listening on 0.0.0.0:{}", cli.port);

    server
        .run_with_graceful_shutdown(
            app,
            async {
                #[cfg(unix)]
                let _ = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .unwrap()
                    .recv()
                    .await;

                #[cfg(not(unix))]
                let _ = tokio::signal::ctrl_c().await;
            },
            None,
        )
        .await?;

    Ok(())
}
