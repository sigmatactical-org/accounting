use std::convert::Infallible;

use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog;
use crate::model::{CreateBill, CreateIntegration, UpdateBill, UpdateIntegration};
use crate::store::StoreError;

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    warp::reply::with_status(
        warp::reply::json(&ErrorBody {
            error: message.into(),
        }),
        status,
    )
    .into_response()
}

fn store_error_status(err: &StoreError) -> StatusCode {
    match err {
        StoreError::BillNotFound | StoreError::IntegrationNotFound => StatusCode::NOT_FOUND,
        StoreError::VendorRequired
        | StoreError::BillDateRequired
        | StoreError::BillNeedsLineItems
        | StoreError::ScanUriRequired
        | StoreError::InvalidQuantity
        | StoreError::IntegrationNameRequired => StatusCode::BAD_REQUEST,
        StoreError::DuplicateIntegrationName => StatusCode::CONFLICT,
        StoreError::Io(_) | StoreError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    list_bills(store.clone())
        .or(get_bill(store.clone()))
        .or(create_bill(store.clone()))
        .or(update_bill(store.clone()))
        .or(delete_bill(store.clone()))
        .or(list_integrations(store.clone()))
        .or(get_integration(store.clone()))
        .or(create_integration(store.clone()))
        .or(update_integration(store.clone()))
        .or(delete_integration(store.clone()))
        .or(list_catalog_skus())
}

fn list_catalog_skus()
-> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("catalog" / "skus")
        .and(warp::path::end())
        .and(warp::get())
        .and_then(|| async {
            let response = match catalog::fetch_skus().await {
                Ok(skus) => warp::reply::json(&skus).into_response(),
                Err(catalog::CatalogError::NotConfigured) => json_error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    catalog::CatalogError::NotConfigured.to_string(),
                ),
                Err(e) => json_error(StatusCode::BAD_GATEWAY, e.to_string()),
            };
            Ok::<_, Rejection>(response)
        })
}

fn list_bills(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("bills")
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let bills = store.lock().await.list_bills();
            Ok::<_, Rejection>(warp::reply::json(&bills))
        })
}

fn get_bill(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            match store.get_bill(&id) {
                Some(bill) => Ok(warp::reply::json(&bill)),
                None => Err(warp::reject::not_found()),
            }
        })
}

fn create_bill(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("bills")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(store)
        .and_then(|input: CreateBill, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.create_bill(input) {
                Ok(bill) => warp::reply::with_status(warp::reply::json(&bill), StatusCode::CREATED)
                    .into_response(),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok::<_, Rejection>(response)
        })
}

fn update_bill(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(store)
        .and_then(
            |id: String, input: UpdateBill, store: SharedStore| async move {
                let mut store = store.lock().await;
                let response = match store.update_bill(&id, input) {
                    Ok(bill) => warp::reply::json(&bill).into_response(),
                    Err(StoreError::BillNotFound) => return Err(warp::reject::not_found()),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                };
                Ok(response)
            },
        )
}

fn delete_bill(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String)
        .and(warp::path::end())
        .and(warp::delete())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.delete_bill(&id) {
                Ok(()) => {
                    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT).into_response()
                }
                Err(StoreError::BillNotFound) => return Err(warp::reject::not_found()),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok(response)
        })
}

fn list_integrations(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("integrations")
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let integrations = store.lock().await.list_integrations();
            Ok::<_, Rejection>(warp::reply::json(&integrations))
        })
}

fn get_integration(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            match store.get_integration(&id) {
                Some(integration) => Ok(warp::reply::json(&integration)),
                None => Err(warp::reject::not_found()),
            }
        })
}

fn create_integration(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("integrations")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(store)
        .and_then(|input: CreateIntegration, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.create_integration(input) {
                Ok(integration) => {
                    warp::reply::with_status(warp::reply::json(&integration), StatusCode::CREATED)
                        .into_response()
                }
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok::<_, Rejection>(response)
        })
}

fn update_integration(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(store)
        .and_then(
            |id: String, input: UpdateIntegration, store: SharedStore| async move {
                let mut store = store.lock().await;
                let response = match store.update_integration(&id, input) {
                    Ok(integration) => warp::reply::json(&integration).into_response(),
                    Err(StoreError::IntegrationNotFound) => return Err(warp::reject::not_found()),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                };
                Ok(response)
            },
        )
}

fn delete_integration(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String)
        .and(warp::path::end())
        .and(warp::delete())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.delete_integration(&id) {
                Ok(()) => {
                    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT).into_response()
                }
                Err(StoreError::IntegrationNotFound) => return Err(warp::reject::not_found()),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok(response)
        })
}
