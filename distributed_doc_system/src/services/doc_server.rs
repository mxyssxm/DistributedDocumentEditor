use actix::prelude::*;
use std::sync::Arc;
use zenoh::Session;
use crate::models::document::Repository;

pub struct DocServer {
    pub(crate) repo: Repository,
    pub(crate) network_session: Arc<Session>,
}

impl DocServer {
    pub fn new(network_session: Arc<Session>) -> Self {
        Self {
            repo: Repository::new(),
            network_session,
        }
    }
}

impl Actor for DocServer {
    type Context = Context<Self>;
}

