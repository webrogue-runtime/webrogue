use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Host};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetWrappConfigResponse {
    /// Successfully retrieved device list
    Status200_SuccessfullyRetrievedDeviceList
    (models::WebrogueConfig)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum MakePeerConnectionResponse {
    /// Successfully made peer connection
    Status200_SuccessfullyMadePeerConnection
    (models::Sdp)
}




/// Default
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Default<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// GetWrappConfig - GET /getWRAPPConfig
    async fn get_wrapp_config(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<GetWrappConfigResponse, E>;

    /// MakePeerConnection - POST /makePeerConnection
    async fn make_peer_connection(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::Sdp,
    ) -> Result<MakePeerConnectionResponse, E>;
}
