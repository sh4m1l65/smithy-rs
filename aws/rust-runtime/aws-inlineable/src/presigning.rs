/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(https://github.com/smithy-lang/smithy-rs/issues/2902): Code generate this documentation so that service-specific examples can be added.
//! Presigned request types and configuration.
//!
//! The [`Client`](crate::Client) is used to create presigned requests. They are made
//! by calling `.presigned()` instead of `.send()` on an operation, and require a
//! [`PresigningConfig`](crate::presigning::PresigningConfig) to provide an expiration time.
//!
//! Only operations that support presigning have the `presigned()` method on them.

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_types::body::SdkBody;
use std::fmt;
use std::time::{Duration, SystemTime};

const ONE_WEEK: Duration = Duration::from_secs(604800);

/// Presigning config values required for creating a presigned request.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct PresigningConfig {
    start_time: SystemTime,
    expires_in: Duration,
}

impl PresigningConfig {
    /// Creates a `PresigningConfig` with the given `expires_in` duration.
    ///
    /// The `expires_in` duration is the total amount of time the presigned request should
    /// be valid for. Other config values are defaulted.
    ///
    /// Credential expiration time takes priority over the `expires_in` value.
    /// If the credentials used to sign the request expire before the presigned request is
    /// set to expire, then the presigned request will become invalid.
    pub fn expires_in(expires_in: Duration) -> Result<PresigningConfig, PresigningConfigError> {
        Self::builder().expires_in(expires_in).build()
    }

    /// Creates a new builder for creating a `PresigningConfig`.
    pub fn builder() -> PresigningConfigBuilder {
        PresigningConfigBuilder::default()
    }

    /// Returns the amount of time the presigned request should be valid for.
    pub fn expires(&self) -> Duration {
        self.expires_in
    }

    /// Returns the start time. The presigned request will be valid between this and the end
    /// time produced by adding the `expires()` value to it.
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }
}

#[derive(Debug)]
enum ErrorKind {
    /// Presigned requests cannot be valid for longer than one week.
    ExpiresInDurationTooLong,

    /// The `PresigningConfig` builder requires a value for `expires_in`.
    ExpiresInRequired,
}

/// `PresigningConfig` build errors.
#[derive(Debug)]
pub struct PresigningConfigError {
    kind: ErrorKind,
}

impl std::error::Error for PresigningConfigError {}

impl fmt::Display for PresigningConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::ExpiresInDurationTooLong => {
                write!(f, "`expires_in` must be no longer than one week")
            }
            ErrorKind::ExpiresInRequired => write!(f, "`expires_in` is required"),
        }
    }
}

impl From<ErrorKind> for PresigningConfigError {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

/// Builder used to create `PresigningConfig`.
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct PresigningConfigBuilder {
    start_time: Option<SystemTime>,
    expires_in: Option<Duration>,
}

impl PresigningConfigBuilder {
    /// Sets the start time for the presigned request.
    ///
    /// The request will start to be valid at this time, and will cease to be valid after
    /// the end time, which can be determined by adding the `expires_in` duration to this
    /// start time. If not specified, this will default to the current time.
    ///
    /// Optional.
    pub fn start_time(mut self, start_time: SystemTime) -> Self {
        self.set_start_time(Some(start_time));
        self
    }

    /// Sets the start time for the presigned request.
    ///
    /// The request will start to be valid at this time, and will cease to be valid after
    /// the end time, which can be determined by adding the `expires_in` duration to this
    /// start time. If not specified, this will default to the current time.
    ///
    /// Optional.
    pub fn set_start_time(&mut self, start_time: Option<SystemTime>) {
        self.start_time = start_time;
    }

    /// Sets how long the request should be valid after the `start_time` (which defaults
    /// to the current time).
    ///
    /// Credential expiration time takes priority over the `expires_in` value.
    /// If the credentials used to sign the request expire before the presigned request is
    /// set to expire, then the presigned request will become invalid.
    ///
    /// Required.
    pub fn expires_in(mut self, expires_in: Duration) -> Self {
        self.set_expires_in(Some(expires_in));
        self
    }

    /// Sets how long the request should be valid after the `start_time` (which defaults
    /// to the current time).
    ///
    /// Credential expiration time takes priority over the `expires_in` value.
    /// If the credentials used to sign the request expire before the presigned request is
    /// set to expire, then the presigned request will become invalid.
    ///
    /// Required.
    pub fn set_expires_in(&mut self, expires_in: Option<Duration>) {
        self.expires_in = expires_in;
    }

    /// Builds the `PresigningConfig`. This will error if `expires_in` is not
    /// given, or if it's longer than one week.
    pub fn build(self) -> Result<PresigningConfig, PresigningConfigError> {
        let expires_in = self.expires_in.ok_or(ErrorKind::ExpiresInRequired)?;
        if expires_in > ONE_WEEK {
            return Err(ErrorKind::ExpiresInDurationTooLong.into());
        }
        Ok(PresigningConfig {
            start_time: self.start_time.unwrap_or_else(
                // This usage is OK—customers can easily override this.
                #[allow(clippy::disallowed_methods)]
                SystemTime::now,
            ),
            expires_in,
        })
    }
}

/// Represents a presigned request. This only includes the HTTP request method, URI, and headers.
///
/// **This struct has conversion convenience functions:**
///
/// - [`PresignedRequest::make_http_02x_request<B>`][Self::make_http_02x_request] returns an [`http::Request<B>`](https://docs.rs/http/0.2.6/http/request/struct.Request.html)
/// - [`PresignedRequest::into`](#impl-From<PresignedRequest>) returns an [`http::request::Builder`](https://docs.rs/http/0.2.6/http/request/struct.Builder.html)
#[non_exhaustive]
pub struct PresignedRequest {
    http_request: HttpRequest,
}

impl Clone for PresignedRequest {
    fn clone(&self) -> Self {
        Self {
            http_request: match self.http_request.try_clone() {
                Some(body) => body,
                None => {
                    unreachable!(
                        "during construction, we replaced the body with `SdkBody::empty()`"
                    )
                }
            },
        }
    }
}

impl PresignedRequest {
    #[allow(dead_code)]
    pub(crate) fn new(inner: HttpRequest) -> Result<Self, BoxError> {
        // throw out the body so we're sure it's cloneable
        let http_request = inner.map(|_body| SdkBody::empty());
        // this should never fail, a presigned request should always be convertible, but better to
        // protect against this potential panic
        let _ = http_request
            .try_clone()
            .expect("must be cloneable, body is empty")
            .try_into_http02x()?;
        Ok(Self { http_request })
    }

    /// Returns the HTTP request method.
    pub fn method(&self) -> &str {
        self.http_request.method()
    }

    /// Returns the HTTP request URI.
    pub fn uri(&self) -> &str {
        self.http_request.uri()
    }

    /// Returns any HTTP headers that need to go along with the request, except for `Host`,
    /// which should be sent based on the endpoint in the URI by the HTTP client rather than
    /// added directly.
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.http_request.headers().iter()
    }

    /// Given a body, produce an `http::Request` from this `PresignedRequest`
    pub fn make_http_02x_request<B>(&self, body: B) -> http::Request<B> {
        self.clone().into_http_02x_request(body)
    }

    /// Converts this `PresignedRequest` directly into an `http` request.
    pub fn into_http_02x_request<B>(self, body: B) -> http::Request<B> {
        self.http_request
            .try_into_http02x()
            .expect("constructor validated convertibility")
            .map(|_req| body)
    }
}

impl fmt::Debug for PresignedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignedRequest")
            .field("method", &self.method())
            .field("uri", &self.uri())
            .field("headers", self.http_request.headers())
            .finish()
    }
}
