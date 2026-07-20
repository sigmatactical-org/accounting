//! Internal JSON API: bills, expenses, integrations, and catalog SKUs.
//!
//! Every route is gated by [`sigma_pg::api::internal_auth`] and built from
//! the generic per-verb helpers below, so each entity contributes only its
//! store calls.

use std::convert::Infallible;
use std::future::Future;

use serde::Serialize;
use serde::de::DeserializeOwned;
use sigma_pg::api::{internal_auth, json_error};
use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog;
use crate::model::{
    CreateBill, CreateExpense, CreateIntegration, CreateReceipt, UpdateBill, UpdateExpense,
    UpdateIntegration,
};
use crate::store::{StoreError, store_error_status};

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    bill_routes(store.clone())
        .or(expense_routes(store.clone()))
        .unify()
        .or(receipt_routes(store.clone()))
        .unify()
        .or(integration_routes(store))
        .unify()
        .or(list_catalog_skus())
        .unify()
}

fn receipt_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    let segment = "receipts";
    list_route(segment, store.clone(), |store: SharedStore| async move {
        store.list_receipts().await
    })
    .or(get_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String| async move { store.get_receipt(&id).await },
    ))
    .unify()
    .or(record_receipt_route(store.clone()))
    .unify()
    .or(delete_route(
        segment,
        store,
        |store: SharedStore, id: String| async move { store.delete_receipt(&id).await },
    ))
    .unify()
}

/// `POST /receipts` — record money received.
///
/// Unlike the other create routes this does not always answer `201`: the
/// store is idempotent on `charge_id`, so a charge that already has a
/// receipt answers `200` with the existing row. Callers retrying a failed
/// push can therefore treat both as success.
fn record_receipt_route(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    warp::path("receipts")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(internal_auth())
        .and(store)
        .and_then(|input: CreateReceipt, store: SharedStore| async move {
            if let Err(e) = crate::orders::validate_order_link(input.order_id.as_deref()).await {
                return Ok::<_, Rejection>(json_error(store_error_status(&e), e.to_string()));
            }
            Ok(match store.record_receipt(input).await {
                Ok((receipt, true)) => json_with_status(&receipt, StatusCode::CREATED),
                Ok((receipt, false)) => json_with_status(&receipt, StatusCode::OK),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            })
        })
}

fn bill_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    let segment = "bills";
    list_route(segment, store.clone(), |store: SharedStore| async move {
        store.list_bills().await
    })
    .or(get_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String| async move { store.get_bill(&id).await },
    ))
    .unify()
    .or(create_route(
        segment,
        store.clone(),
        |store: SharedStore, input: CreateBill| async move {
            crate::orders::validate_order_link(input.order_id.as_deref()).await?;
            store.create_bill(input).await
        },
    ))
    .unify()
    .or(update_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String, input: UpdateBill| async move {
            crate::orders::validate_order_link(input.order_id.as_deref()).await?;
            store.update_bill(&id, input).await
        },
    ))
    .unify()
    .or(delete_route(
        segment,
        store,
        |store: SharedStore, id: String| async move { store.delete_bill(&id).await },
    ))
    .unify()
}

fn expense_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    let segment = "expenses";
    list_route(segment, store.clone(), |store: SharedStore| async move {
        store.list_expenses().await
    })
    .or(get_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String| async move { store.get_expense(&id).await },
    ))
    .unify()
    .or(create_route(
        segment,
        store.clone(),
        |store: SharedStore, input: CreateExpense| async move {
            crate::orders::validate_order_link(input.order_id.as_deref()).await?;
            store.create_expense(input).await
        },
    ))
    .unify()
    .or(update_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String, input: UpdateExpense| async move {
            crate::orders::validate_order_link(input.order_id.as_deref()).await?;
            store.update_expense(&id, input).await
        },
    ))
    .unify()
    .or(delete_route(
        segment,
        store,
        |store: SharedStore, id: String| async move { store.delete_expense(&id).await },
    ))
    .unify()
}

fn integration_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    let segment = "integrations";
    list_route(segment, store.clone(), |store: SharedStore| async move {
        store.list_integrations().await
    })
    .or(get_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String| async move { store.get_integration(&id).await },
    ))
    .unify()
    .or(create_route(
        segment,
        store.clone(),
        |store: SharedStore, input: CreateIntegration| async move {
            store.create_integration(input).await
        },
    ))
    .unify()
    .or(update_route(
        segment,
        store.clone(),
        |store: SharedStore, id: String, input: UpdateIntegration| async move {
            store.update_integration(&id, input).await
        },
    ))
    .unify()
    .or(delete_route(
        segment,
        store,
        |store: SharedStore, id: String| async move { store.delete_integration(&id).await },
    ))
    .unify()
}

/// `GET /{segment}` — the whole collection as JSON.
fn list_route<T, F, Fut>(
    segment: &'static str,
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    list: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
where
    T: Serialize + Send,
    F: Fn(SharedStore) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>, StoreError>> + Send,
{
    warp::path(segment)
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(move |store: SharedStore| {
            let list = list.clone();
            async move {
                match list(store).await {
                    Ok(items) => Ok(warp::reply::json(&items).into_response()),
                    Err(e) => Err(store_rejection(&e)),
                }
            }
        })
}

/// `GET /{segment}/{id}` — one entity as JSON, 404 when missing.
fn get_route<T, F, Fut>(
    segment: &'static str,
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    get: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
where
    T: Serialize + Send,
    F: Fn(SharedStore, String) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<T>, StoreError>> + Send,
{
    warp::path(segment)
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(move |id: String, store: SharedStore| {
            let get = get.clone();
            async move {
                match get(store, id).await {
                    Ok(Some(entity)) => Ok(warp::reply::json(&entity).into_response()),
                    Ok(None) => Err(warp::reject::not_found()),
                    Err(e) => Err(store_rejection(&e)),
                }
            }
        })
}

/// `POST /{segment}` — create from a JSON body, 201 on success.
fn create_route<I, T, F, Fut>(
    segment: &'static str,
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    create: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
where
    I: DeserializeOwned + Send + 'static,
    T: Serialize + Send,
    F: Fn(SharedStore, I) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<T, StoreError>> + Send,
{
    warp::path(segment)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(internal_auth())
        .and(store)
        .and_then(move |input: I, store: SharedStore| {
            let create = create.clone();
            async move {
                Ok::<_, Rejection>(match create(store, input).await {
                    Ok(entity) => json_with_status(&entity, StatusCode::CREATED),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                })
            }
        })
}

/// `PUT /{segment}/{id}` — replace from a JSON body, 404 when missing.
fn update_route<I, T, F, Fut>(
    segment: &'static str,
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    update: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
where
    I: DeserializeOwned + Send + 'static,
    T: Serialize + Send,
    F: Fn(SharedStore, String, I) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<T, StoreError>> + Send,
{
    warp::path(segment)
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(internal_auth())
        .and(store)
        .and_then(move |id: String, input: I, store: SharedStore| {
            let update = update.clone();
            async move {
                match update(store, id, input).await {
                    Ok(entity) => Ok(warp::reply::json(&entity).into_response()),
                    Err(e) if e.is_not_found() => Err(warp::reject::not_found()),
                    Err(e) => Ok(json_error(store_error_status(&e), e.to_string())),
                }
            }
        })
}

/// `DELETE /{segment}/{id}` — 204 on success, 404 when missing.
fn delete_route<F, Fut>(
    segment: &'static str,
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    delete: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
where
    F: Fn(SharedStore, String) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<(), StoreError>> + Send,
{
    warp::path(segment)
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::delete())
        .and(internal_auth())
        .and(store)
        .and_then(move |id: String, store: SharedStore| {
            let delete = delete.clone();
            async move {
                match delete(store, id).await {
                    Ok(()) => Ok(
                        warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT)
                            .into_response(),
                    ),
                    Err(e) if e.is_not_found() => Err(warp::reject::not_found()),
                    Err(e) => Ok(json_error(store_error_status(&e), e.to_string())),
                }
            }
        })
}

fn list_catalog_skus()
-> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("catalog" / "skus")
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and_then(|| async {
            Ok::<_, Rejection>(match catalog::fetch_skus().await {
                Ok(skus) => warp::reply::json(skus.as_ref()).into_response(),
                Err(e @ catalog::CatalogError::NotConfigured) => {
                    json_error(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
                }
                Err(e) => json_error(StatusCode::BAD_GATEWAY, e.to_string()),
            })
        })
}

fn json_with_status<T: Serialize>(value: &T, status: StatusCode) -> Response {
    warp::reply::with_status(warp::reply::json(value), status).into_response()
}

/// Read failures have no JSON body to carry a message, so they surface as
/// the themed 404/500 pages produced by the shared rejection handler.
fn store_rejection(err: &StoreError) -> Rejection {
    if err.is_not_found() {
        warp::reject::not_found()
    } else {
        warp::reject::reject()
    }
}
