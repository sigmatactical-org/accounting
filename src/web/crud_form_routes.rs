//! [`CrudFormRoutes`].

use std::convert::Infallible;
use std::future::Future;

use serde::de::DeserializeOwned;
use warp::reply::Response;
use warp::{Filter, Rejection};

use crate::SharedStore;

use super::{AdminGate, cookie_filter, require_admin};

/// The five HTML-form routes every accounting entity has:
/// `GET /{segment}/new`, `POST /{segment}`, `GET /{segment}/{id}/edit`,
/// `POST /{segment}/{id}/edit`, and `POST /{segment}/{id}/delete`.
///
/// Each handler owns the entity-specific work (rendering, validation, store
/// calls); this type owns the shared warp wiring — paths, methods, form-body
/// extraction, and store injection — so the entities don't repeat it.
pub(crate) struct CrudFormRoutes<NewPage, Create, EditPage, Update, Delete> {
    /// URL segment, e.g. `"bills"`.
    pub(crate) segment: &'static str,
    pub(crate) new_page: NewPage,
    pub(crate) create: Create,
    pub(crate) edit_page: EditPage,
    pub(crate) update: Update,
    pub(crate) delete: Delete,
}

impl<NewPage, NewFut, Create, EditPage, EditFut, Update, Delete, DeleteFut>
    CrudFormRoutes<NewPage, Create, EditPage, Update, Delete>
where
    NewPage: Fn() -> NewFut + Clone + Send + Sync + 'static,
    NewFut: Future<Output = Result<Response, Rejection>> + Send,
    EditPage: Fn(SharedStore, String) -> EditFut + Clone + Send + Sync + 'static,
    EditFut: Future<Output = Result<Response, Rejection>> + Send,
    Delete: Fn(SharedStore, String) -> DeleteFut + Clone + Send + Sync + 'static,
    DeleteFut: Future<Output = Result<Response, Rejection>> + Send,
{
    /// Wire the handlers onto their routes.
    pub(crate) fn build<Form, CreateFut, UpdateFut>(
        self,
        store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    ) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + 'static
    where
        Form: DeserializeOwned + Send + 'static,
        Create: Fn(SharedStore, Form) -> CreateFut + Clone + Send + Sync + 'static,
        CreateFut: Future<Output = Result<Response, Rejection>> + Send,
        Update: Fn(SharedStore, String, Form) -> UpdateFut + Clone + Send + Sync + 'static,
        UpdateFut: Future<Output = Result<Response, Rejection>> + Send,
    {
        let Self {
            segment,
            new_page,
            create,
            edit_page,
            update,
            delete,
        } = self;

        let new_return_path = format!("/{segment}/new");

        let new_page_route = warp::path(segment)
            .and(warp::path("new"))
            .and(warp::path::end())
            .and(warp::get())
            .and(cookie_filter())
            .and_then({
                let new_return_path = new_return_path.clone();
                move |cookie: Option<String>| {
                    let new_page = new_page.clone();
                    let new_return_path = new_return_path.clone();
                    async move {
                        match require_admin(cookie.as_deref(), &new_return_path).await {
                            AdminGate::Allow => {}
                            AdminGate::SignIn(resp) => return Ok(resp),
                            AdminGate::Deny => return Err(warp::reject::not_found()),
                        }
                        new_page().await
                    }
                }
            });

        let create_route = warp::path(segment)
            .and(warp::path::end())
            .and(warp::post())
            .and(cookie_filter())
            .and(warp::body::form::<Form>())
            .and(store.clone())
            .and_then({
                let new_return_path = new_return_path.clone();
                move |cookie: Option<String>, form: Form, store: SharedStore| {
                    let create = create.clone();
                    let new_return_path = new_return_path.clone();
                    async move {
                        match require_admin(cookie.as_deref(), &new_return_path).await {
                            AdminGate::Allow => {}
                            AdminGate::SignIn(resp) => return Ok(resp),
                            AdminGate::Deny => return Err(warp::reject::not_found()),
                        }
                        create(store, form).await
                    }
                }
            });

        let edit_page_route = warp::path(segment)
            .and(warp::path::param::<String>())
            .and(warp::path("edit"))
            .and(warp::path::end())
            .and(warp::get())
            .and(cookie_filter())
            .and(store.clone())
            .and_then(move |id: String, cookie: Option<String>, store: SharedStore| {
                let edit_page = edit_page.clone();
                async move {
                    let return_path = format!("/{segment}/{id}/edit");
                    match require_admin(cookie.as_deref(), &return_path).await {
                        AdminGate::Allow => {}
                        AdminGate::SignIn(resp) => return Ok(resp),
                        AdminGate::Deny => return Err(warp::reject::not_found()),
                    }
                    edit_page(store, id).await
                }
            });

        let update_route = warp::path(segment)
            .and(warp::path::param::<String>())
            .and(warp::path("edit"))
            .and(warp::path::end())
            .and(warp::post())
            .and(cookie_filter())
            .and(warp::body::form::<Form>())
            .and(store.clone())
            .and_then(move |id: String, cookie: Option<String>, form: Form, store: SharedStore| {
                let update = update.clone();
                async move {
                    let return_path = format!("/{segment}/{id}/edit");
                    match require_admin(cookie.as_deref(), &return_path).await {
                        AdminGate::Allow => {}
                        AdminGate::SignIn(resp) => return Ok(resp),
                        AdminGate::Deny => return Err(warp::reject::not_found()),
                    }
                    update(store, id, form).await
                }
            });

        let delete_route = warp::path(segment)
            .and(warp::path::param::<String>())
            .and(warp::path("delete"))
            .and(warp::path::end())
            .and(warp::post())
            .and(cookie_filter())
            .and(store)
            .and_then(move |id: String, cookie: Option<String>, store: SharedStore| {
                let delete = delete.clone();
                async move {
                    match require_admin(cookie.as_deref(), "/").await {
                        AdminGate::Allow => {}
                        AdminGate::SignIn(resp) => return Ok(resp),
                        AdminGate::Deny => return Err(warp::reject::not_found()),
                    }
                    delete(store, id).await
                }
            });

        new_page_route
            .or(create_route)
            .unify()
            .or(edit_page_route)
            .unify()
            .or(update_route)
            .unify()
            .or(delete_route)
            .unify()
    }
}
