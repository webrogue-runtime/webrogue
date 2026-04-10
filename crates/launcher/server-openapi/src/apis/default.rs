use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::CookieJar;
use bytes::Bytes;
use headers::Host;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ConnectDeviceResponse {
    /// Success
    Status200_Success
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetDeviceNameResponse {
    /// Success
    Status200_Success
    (models::GetDeviceNameResponse)
}




/// Default
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Default<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// Connect device for debgging.
    ///
    /// ConnectDevice - POST /connect_device
    async fn connect_device(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::ConnectDeviceRequest,
    ) -> Result<ConnectDeviceResponse, E>;

    /// Get device name.
    ///
    /// GetDeviceName - GET /get_device_name
    async fn get_device_name(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<GetDeviceNameResponse, E>;
}
