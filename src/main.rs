use std::{net::Ipv4Addr, path::PathBuf, str::FromStr};

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

mod providers;
use providers::IpgeoProvider;

use crate::providers::{HerrbischoffProvider, Ip2locationProvider};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env, default_value = "3000", help = "Port to listen on")]
    port: u16,
    #[clap(long, env, help = "Path to the country-ip-blocks repository")]
    herrbischoff_path: Option<PathBuf>,
    #[clap(
        long,
        env,
        help = "Path to the IP2Location LITE CSV-formatted database"
    )]
    ip2location_db: Option<PathBuf>,
}

#[derive(Debug)]
struct Api {
    provider: IpgeoProvider,
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
    fn new(provider: IpgeoProvider) -> Self {
        Self { provider }
    }
}

#[OpenApi]
impl Api {
    #[oai(path = "/ipv4/:ip_address", method = "get")]
    /// Gets the two-letter ISO 3166 country code associated with the IPv4 address
    async fn get_ipv4(&self, ip_address: Path<String>) -> Result<Json<IpGeolocation>, ApiError> {
        let ip_address =
            Ipv4Addr::from_str(&ip_address.0).map_err(|_| ApiError::InvalidIpAddress)?;

        match self.provider.get_ipv4_country(&ip_address) {
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

    let provider = match (cli.herrbischoff_path, cli.ip2location_db) {
        (Some(herrbischoff_path), None) => {
            IpgeoProvider::Herrbischoff(HerrbischoffProvider::from_repo(&herrbischoff_path)?)
        }
        (None, Some(ip2location_db)) => {
            IpgeoProvider::Ip2location(Ip2locationProvider::from_db(&ip2location_db)?)
        }
        (None, None) => anyhow::bail!("no valid IP geolocation database source provided"),
        _ => anyhow::bail!("one and only one source should be provided"),
    };

    let api = Api::new(provider);
    let api_service = OpenApiService::new(api, "ipgeod", env!("CARGO_PKG_VERSION"));

    let app = Route::new()
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/swagger", api_service.swagger_ui())
        .nest("/", api_service.with(Cors::new()).with(Tracing));

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
