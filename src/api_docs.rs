// src/api_docs.rs
use crate::models::{Earthquake, ListResponse, Pagination};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::list_earthquakes,
        crate::routes::get_earthquake,
    ),
    components(
        schemas(Earthquake, ListResponse::<Earthquake>, Pagination)
    ),
    info(
        title = "Earthquake Monitor API",
        description = "API for ingesting and serving earthquake events (USGS)",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;
