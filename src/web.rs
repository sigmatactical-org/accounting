use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog::{self, CatalogSku};
use crate::model::{BillForm, IntegrationForm};
use crate::store::StoreError;
use crate::templates::{self, BillFormValues, IntegrationFormValues};

pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    index_page(store.clone())
        .or(new_bill_page(store.clone()))
        .or(create_bill_form(store.clone()))
        .or(edit_bill_page(store.clone()))
        .or(update_bill_form(store.clone()))
        .or(delete_bill_form(store.clone()))
        .or(new_integration_page(store.clone()))
        .or(create_integration_form(store.clone()))
        .or(edit_integration_page(store.clone()))
        .or(update_integration_form(store.clone()))
        .or(delete_integration_form(store))
}

async fn fetch_catalog_skus() -> (Vec<CatalogSku>, Option<String>) {
    match catalog::fetch_skus().await {
        Ok(skus) => (skus, None),
        Err(catalog::CatalogError::NotConfigured) => (Vec::new(), None),
        Err(e) => (Vec::new(), Some(format!("Catalog unavailable: {e}"))),
    }
}

fn index_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path::end()
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let store = store.lock().await;
            let (catalog_skus, catalog_notice) = fetch_catalog_skus().await;
            templates::render_index_html(
                store.list_bills(),
                store.list_integrations(),
                catalog_skus,
                catalog_notice,
                None,
            )
            .map(warp::reply::html)
            .map_err(|_| warp::reject::not_found())
        })
}

fn new_bill_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("bills")
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|_store: SharedStore| async move {
            let (catalog_skus, _) = fetch_catalog_skus().await;
            templates::render_bill_form_html(catalog_skus, None, None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn create_bill_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("bills")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(|form: BillForm, store: SharedStore| async move {
            let mut store = store.lock().await;
            let values = bill_form_to_values(&form);
            let (catalog_skus, _) = fetch_catalog_skus().await;
            let response = match form.into_create() {
                Ok(input) => match store.create_bill(input) {
                    Ok(_) => {
                        warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response()
                    }
                    Err(e) => render_bill_form_error(catalog_skus, None, values, e),
                },
                Err(e) => render_bill_form_error(catalog_skus, None, values, invalid_input(e)),
            };
            Ok::<_, Rejection>(response)
        })
}

fn edit_bill_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String / "edit")
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            let Some(bill) = store.get_bill(&id) else {
                return Err(warp::reject::not_found());
            };
            let (catalog_skus, _) = fetch_catalog_skus().await;
            templates::render_bill_form_html(catalog_skus, Some(bill), None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn update_bill_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String / "edit")
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |id: String, form: BillForm, store: SharedStore| async move {
                let mut store = store.lock().await;
                let values = bill_form_to_values(&form);
                let (catalog_skus, _) = fetch_catalog_skus().await;
                let response = match form.into_update() {
                    Ok(input) => match store.update_bill(&id, input) {
                        Ok(_) => warp::redirect::redirect(warp::http::Uri::from_static("/"))
                            .into_response(),
                        Err(e) => {
                            let bill = store.get_bill(&id);
                            render_bill_form_error(catalog_skus, bill, values, e)
                        }
                    },
                    Err(e) => {
                        let bill = store.get_bill(&id);
                        render_bill_form_error(catalog_skus, bill, values, invalid_input(e))
                    }
                };
                Ok::<_, Rejection>(response)
            },
        )
}

fn delete_bill_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("bills" / String / "delete")
        .and(warp::post())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            match store.delete_bill(&id) {
                Ok(()) => {
                    Ok(warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response())
                }
                Err(StoreError::BillNotFound) => Err(warp::reject::not_found()),
                Err(e) => {
                    let (catalog_skus, catalog_notice) = fetch_catalog_skus().await;
                    templates::render_index_html(
                        store.list_bills(),
                        store.list_integrations(),
                        catalog_skus,
                        catalog_notice,
                        Some(format!("Delete failed: {e}")),
                    )
                    .map(|html| warp::reply::html(html).into_response())
                    .map_err(|_| warp::reject::not_found())
                }
            }
        })
}

fn new_integration_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("integrations")
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|_store: SharedStore| async move {
            templates::render_integration_form_html(None, None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn create_integration_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("integrations")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(|form: IntegrationForm, store: SharedStore| async move {
            let mut store = store.lock().await;
            let values = integration_form_to_values(&form);
            let response = match form.into_create() {
                Ok(input) => match store.create_integration(input) {
                    Ok(_) => {
                        warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response()
                    }
                    Err(e) => render_integration_form_error(None, values, e),
                },
                Err(e) => render_integration_form_error(None, values, invalid_input(e)),
            };
            Ok::<_, Rejection>(response)
        })
}

fn edit_integration_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String / "edit")
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            let Some(integration) = store.get_integration(&id) else {
                return Err(warp::reject::not_found());
            };
            templates::render_integration_form_html(Some(integration), None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn update_integration_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String / "edit")
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |id: String, form: IntegrationForm, store: SharedStore| async move {
                let mut store = store.lock().await;
                let values = integration_form_to_values(&form);
                let response = match form.into_update() {
                    Ok(input) => match store.update_integration(&id, input) {
                        Ok(_) => warp::redirect::redirect(warp::http::Uri::from_static("/"))
                            .into_response(),
                        Err(e) => {
                            let integration = store.get_integration(&id);
                            render_integration_form_error(integration, values, e)
                        }
                    },
                    Err(e) => {
                        let integration = store.get_integration(&id);
                        render_integration_form_error(integration, values, invalid_input(e))
                    }
                };
                Ok::<_, Rejection>(response)
            },
        )
}

fn delete_integration_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("integrations" / String / "delete")
        .and(warp::post())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            match store.delete_integration(&id) {
                Ok(()) => {
                    Ok(warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response())
                }
                Err(StoreError::IntegrationNotFound) => Err(warp::reject::not_found()),
                Err(e) => {
                    let (catalog_skus, catalog_notice) = fetch_catalog_skus().await;
                    templates::render_index_html(
                        store.list_bills(),
                        store.list_integrations(),
                        catalog_skus,
                        catalog_notice,
                        Some(format!("Delete failed: {e}")),
                    )
                    .map(|html| warp::reply::html(html).into_response())
                    .map_err(|_| warp::reject::not_found())
                }
            }
        })
}

fn bill_form_to_values(form: &BillForm) -> BillFormValues {
    BillFormValues {
        kind: form.kind.clone(),
        status: form.status.clone(),
        vendor: form.vendor.clone(),
        invoice_number: form.invoice_number.clone(),
        bill_date: form.bill_date.clone(),
        due_date: form.due_date.clone(),
        currency: form.currency.clone(),
        line_items: form.line_items.clone(),
        scan_uri: form.scan_uri.clone(),
        notes: form.notes.clone(),
    }
}

fn integration_form_to_values(form: &IntegrationForm) -> IntegrationFormValues {
    IntegrationFormValues {
        name: form.name.clone(),
        provider: form.provider.clone(),
        enabled: form.enabled.is_some(),
        external_account_id: form.external_account_id.clone(),
        webhook_url: form.webhook_url.clone(),
        notes: form.notes.clone(),
    }
}

fn invalid_input(message: String) -> StoreError {
    StoreError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message,
    ))
}

fn render_bill_form_error(
    catalog_skus: Vec<CatalogSku>,
    bill: Option<crate::model::Bill>,
    values: BillFormValues,
    err: StoreError,
) -> warp::reply::Response {
    let message = err.to_string();
    match templates::render_bill_form_html_with_values(catalog_skus, bill, Some(message), values) {
        Ok(html) => warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
            .into_response(),
        Err(_) => warp::reply::with_status(warp::reply(), StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    }
}

fn render_integration_form_error(
    integration: Option<crate::model::Integration>,
    values: IntegrationFormValues,
    err: StoreError,
) -> warp::reply::Response {
    let message = err.to_string();
    match templates::render_integration_form_html_with_values(integration, Some(message), values) {
        Ok(html) => warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
            .into_response(),
        Err(_) => warp::reply::with_status(warp::reply(), StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    }
}
