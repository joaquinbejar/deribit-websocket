//! Mock-server integration test modules.
//!
//! Each submodule exercises one flow from issue #53 against the shared
//! mock server and client helpers in [`helpers`].

pub(crate) mod helpers;

mod auth;
mod id_matching;
mod notifications;
mod reconnect;
mod subscriptions;
mod timeout;
