use actix::prelude::*;
use std::sync::Arc;
use zenoh::Session;
use crate::models::document::Repository;

pub struct DocServer {
    pub(crate) repo: Repository,
    pub(crate) zenoh_session: Arc<Session>,
}

impl DocServer {
    pub fn new(zenoh_session: Arc<Session>) -> Self {
        Self {
            repo: Repository::new(),
            zenoh_session,
        }
    }
}

impl Actor for DocServer {
    type Context = Context<Self>;
}

