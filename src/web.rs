//! HTML UI: the index page plus the new/create/edit/update/delete form
//! routes for bills, expenses, and integrations, all built from
//! [`CrudFormRoutes`].

mod crud_form_routes;
use crud_form_routes::CrudFormRoutes;

use std::convert::Infallible;
use std::sync::Arc;

use crate::store::StoreError;
use warp::http::{StatusCode, Uri};
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog::{self, CatalogSku};
use crate::model::{
    Bill, BillForm, CreateBill, CreateExpense, Expense, ExpenseForm, Integration, IntegrationForm,
    UpdateBill, UpdateExpense,
};
use crate::templates::{self, BillFormValues, ExpenseFormValues, IntegrationFormValues};

/// Catalog SKUs plus a banner when the catalog service is unreachable.
type CatalogSkus = (Arc<Vec<CatalogSku>>, Option<String>);

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    index_page(store.clone())
        .or(bill_form_routes(store.clone()))
        .unify()
        .or(expense_form_routes(store.clone()))
        .unify()
        .or(receipt_routes(store.clone()))
        .unify()
        .or(integration_form_routes(store))
        .unify()
}

/// Receipts are recorded by the cart and by reconcile, never typed in by
/// hand, so there is no create/edit form — only the reconcile action and a
/// delete escape hatch.
fn receipt_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    reconcile_receipts_form(store.clone())
        .or(delete_receipt_form(store))
        .unify()
}

/// `POST /receipts/reconcile` — sweep the payments charge log into receipts.
fn reconcile_receipts_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("receipts" / "reconcile")
        .and(warp::post())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let message = match crate::payments::reconcile_receipts(&store).await {
                Ok(outcome) => format!(
                    "Reconcile: {} successful charge(s) seen, {} new receipt(s) recorded, \
                     {} already recorded.",
                    outcome.charges_seen, outcome.created, outcome.already_recorded
                ),
                Err(e) => format!("Reconcile failed: {e}"),
            };
            render_index(&store, Some(message)).await
        })
}

fn delete_receipt_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("receipts" / String / "delete")
        .and(warp::post())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            match store.delete_receipt(&id).await {
                Ok(()) => Ok(redirect_to_index()),
                Err(e) if e.is_not_found() => Err(warp::reject::not_found()),
                Err(e) => render_index(&store, Some(format!("Delete failed: {e}"))).await,
            }
        })
}

/// Catalog SKUs for a page render. The shared client caches per process with
/// a short TTL, so this is usually a cache hit rather than an HTTP call. A
/// missing configuration is normal (no catalog links); an unreachable
/// catalog shows a banner.
async fn fetch_catalog_skus() -> CatalogSkus {
    match catalog::fetch_skus().await {
        Ok(skus) => (skus, None),
        Err(catalog::CatalogError::NotConfigured) => (Arc::new(Vec::new()), None),
        Err(e) => (
            Arc::new(Vec::new()),
            Some(format!("Catalog unavailable: {e}")),
        ),
    }
}

fn index_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    warp::path::end()
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move { render_index(&store, None).await })
}

/// Render the index, optionally with a status message (delete-failure
/// fallback).
async fn render_index(store: &SharedStore, message: Option<String>) -> Result<Response, Rejection> {
    let (catalog_skus, catalog_notice) = fetch_catalog_skus().await;
    let bills = store.list_bills().await.map_err(page_rejection)?;
    let expenses = store.list_expenses().await.map_err(page_rejection)?;
    let receipts = store.list_receipts().await.map_err(page_rejection)?;
    let integrations = store.list_integrations().await.map_err(page_rejection)?;
    html_page(templates::render_index_html(
        bills,
        expenses,
        receipts,
        integrations,
        &catalog_skus,
        catalog_notice,
        message,
    ))
}

fn bill_form_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    CrudFormRoutes {
        segment: "bills",
        new_page: || async {
            let (catalog_skus, _) = fetch_catalog_skus().await;
            html_page(templates::render_bill_form_html(&catalog_skus, None, None))
        },
        create: |store: SharedStore, form: BillForm| async move {
            match form.into_create() {
                Ok(input) => match create_bill(&store, input).await {
                    Ok(()) => Ok(redirect_to_index()),
                    Err(e) => Ok(bill_form_error(None, None, &e.to_string()).await),
                },
                Err((message, form)) => {
                    Ok(bill_form_error(None, Some(form.into()), &message).await)
                }
            }
        },
        edit_page: |store: SharedStore, id: String| async move {
            let bill = fetch_or_404(store.get_bill(&id).await)?;
            let (catalog_skus, _) = fetch_catalog_skus().await;
            html_page(templates::render_bill_form_html(
                &catalog_skus,
                Some(bill),
                None,
            ))
        },
        update: |store: SharedStore, id: String, form: BillForm| async move {
            match form.into_update() {
                Ok(input) => match update_bill(&store, &id, input).await {
                    Ok(()) => Ok(redirect_to_index()),
                    Err(e) => {
                        let bill = store.get_bill(&id).await.ok().flatten();
                        Ok(bill_form_error(bill, None, &e.to_string()).await)
                    }
                },
                Err((message, form)) => {
                    let bill = store.get_bill(&id).await.ok().flatten();
                    Ok(bill_form_error(bill, Some(form.into()), &message).await)
                }
            }
        },
        delete: |store: SharedStore, id: String| async move {
            delete_or_message(&store, store.delete_bill(&id).await).await
        },
    }
    .build::<BillForm, _, _>(store)
}

fn expense_form_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    CrudFormRoutes {
        segment: "expenses",
        new_page: || async { html_page(templates::render_expense_form_html(None, None)) },
        create: |store: SharedStore, form: ExpenseForm| async move {
            match form.into_create() {
                Ok(input) => match create_expense(&store, input).await {
                    Ok(()) => Ok(redirect_to_index()),
                    Err(e) => Ok(expense_form_error(None, None, &e.to_string())),
                },
                Err((message, form)) => Ok(expense_form_error(None, Some(form.into()), &message)),
            }
        },
        edit_page: |store: SharedStore, id: String| async move {
            let expense = fetch_or_404(store.get_expense(&id).await)?;
            html_page(templates::render_expense_form_html(Some(expense), None))
        },
        update: |store: SharedStore, id: String, form: ExpenseForm| async move {
            match form.into_update() {
                Ok(input) => match update_expense(&store, &id, input).await {
                    Ok(()) => Ok(redirect_to_index()),
                    Err(e) => {
                        let expense = store.get_expense(&id).await.ok().flatten();
                        Ok(expense_form_error(expense, None, &e.to_string()))
                    }
                },
                Err((message, form)) => {
                    let expense = store.get_expense(&id).await.ok().flatten();
                    Ok(expense_form_error(expense, Some(form.into()), &message))
                }
            }
        },
        delete: |store: SharedStore, id: String| async move {
            delete_or_message(&store, store.delete_expense(&id).await).await
        },
    }
    .build::<ExpenseForm, _, _>(store)
}

fn integration_form_routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static {
    CrudFormRoutes {
        segment: "integrations",
        new_page: || async { html_page(templates::render_integration_form_html(None, None)) },
        create: |store: SharedStore, form: IntegrationForm| async move {
            match form.into_create() {
                Ok(input) => match store.create_integration(input).await {
                    Ok(_) => Ok(redirect_to_index()),
                    Err(e) => Ok(integration_form_error(None, None, &e.to_string())),
                },
                Err((message, form)) => {
                    Ok(integration_form_error(None, Some(form.into()), &message))
                }
            }
        },
        edit_page: |store: SharedStore, id: String| async move {
            let integration = fetch_or_404(store.get_integration(&id).await)?;
            html_page(templates::render_integration_form_html(
                Some(integration),
                None,
            ))
        },
        update: |store: SharedStore, id: String, form: IntegrationForm| async move {
            match form.into_update() {
                Ok(input) => match store.update_integration(&id, input).await {
                    Ok(_) => Ok(redirect_to_index()),
                    Err(e) => {
                        let integration = store.get_integration(&id).await.ok().flatten();
                        Ok(integration_form_error(integration, None, &e.to_string()))
                    }
                },
                Err((message, form)) => {
                    let integration = store.get_integration(&id).await.ok().flatten();
                    Ok(integration_form_error(
                        integration,
                        Some(form.into()),
                        &message,
                    ))
                }
            }
        },
        delete: |store: SharedStore, id: String| async move {
            delete_or_message(&store, store.delete_integration(&id).await).await
        },
    }
    .build::<IntegrationForm, _, _>(store)
}

async fn create_bill(store: &SharedStore, input: CreateBill) -> Result<(), StoreError> {
    crate::orders::validate_order_link(input.order_id.as_deref()).await?;
    store.create_bill(input).await.map(|_| ())
}

async fn update_bill(store: &SharedStore, id: &str, input: UpdateBill) -> Result<(), StoreError> {
    crate::orders::validate_order_link(input.order_id.as_deref()).await?;
    store.update_bill(id, input).await.map(|_| ())
}

async fn create_expense(store: &SharedStore, input: CreateExpense) -> Result<(), StoreError> {
    crate::orders::validate_order_link(input.order_id.as_deref()).await?;
    store.create_expense(input).await.map(|_| ())
}

async fn update_expense(
    store: &SharedStore,
    id: &str,
    input: UpdateExpense,
) -> Result<(), StoreError> {
    crate::orders::validate_order_link(input.order_id.as_deref()).await?;
    store.update_expense(id, input).await.map(|_| ())
}

/// Redirect back to the index after a successful mutation.
fn redirect_to_index() -> Response {
    warp::redirect::redirect(Uri::from_static("/")).into_response()
}

/// A missing row is a themed 404; a failed read is a themed 500.
fn fetch_or_404<T>(fetched: Result<Option<T>, StoreError>) -> Result<T, Rejection> {
    match fetched {
        Ok(Some(entity)) => Ok(entity),
        Ok(None) => Err(warp::reject::not_found()),
        Err(e) => Err(page_rejection(e)),
    }
}

/// Deleting a missing row is a themed 404; any other failure re-renders the
/// index with the reason.
async fn delete_or_message(
    store: &SharedStore,
    deleted: Result<(), StoreError>,
) -> Result<Response, Rejection> {
    match deleted {
        Ok(()) => Ok(redirect_to_index()),
        Err(e) if e.is_not_found() => Err(warp::reject::not_found()),
        Err(e) => render_index(store, Some(format!("Delete failed: {e}"))).await,
    }
}

fn page_rejection(err: StoreError) -> Rejection {
    if err.is_not_found() {
        warp::reject::not_found()
    } else {
        warp::reject::reject()
    }
}

fn html_page(rendered: Result<String, askama::Error>) -> Result<Response, Rejection> {
    rendered
        .map(|html| warp::reply::html(html).into_response())
        .map_err(|_| warp::reject::reject())
}

/// 400 with the re-rendered form, or a bare 500 when even that render fails.
fn form_error_response(rendered: Result<String, askama::Error>) -> Response {
    match rendered {
        Ok(html) => warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
            .into_response(),
        Err(_) => warp::reply::with_status(warp::reply(), StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    }
}

/// Re-render the bill form with `message`. `values` carries the submitted
/// fields back when the submission itself was rejected; without it the form
/// falls back to the stored bill (or blank defaults).
async fn bill_form_error(
    bill: Option<Bill>,
    values: Option<BillFormValues>,
    message: &str,
) -> Response {
    let (catalog_skus, _) = fetch_catalog_skus().await;
    let error = Some(message.to_string());
    form_error_response(match values {
        Some(values) => {
            templates::render_bill_form_html_with_values(&catalog_skus, bill, error, values)
        }
        None => templates::render_bill_form_html(&catalog_skus, bill, error),
    })
}

fn expense_form_error(
    expense: Option<Expense>,
    values: Option<ExpenseFormValues>,
    message: &str,
) -> Response {
    let error = Some(message.to_string());
    form_error_response(match values {
        Some(values) => templates::render_expense_form_html_with_values(expense, error, values),
        None => templates::render_expense_form_html(expense, error),
    })
}

fn integration_form_error(
    integration: Option<Integration>,
    values: Option<IntegrationFormValues>,
    message: &str,
) -> Response {
    let error = Some(message.to_string());
    form_error_response(match values {
        Some(values) => {
            templates::render_integration_form_html_with_values(integration, error, values)
        }
        None => templates::render_integration_form_html(integration, error),
    })
}
