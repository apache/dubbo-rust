/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{error::Error, fmt};

use http::HeaderValue;

pub const GRPC_STATUS: &str = "grpc-status";
pub const GRPC_MESSAGE: &str = "grpc-message";

/// error codes for grpc APIs
/// https://github.com/googleapis/googleapis/blob/master/google/rpc/code.proto
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    // Not an error; returned on success
    //
    // HTTP Mapping: 200 OK
    Ok = 0,

    // The operation was cancelled, typically by the caller.
    //
    // HTTP Mapping: 499 Client Closed Request
    Cancelled = 1,

    // Unknown error.  For example, this error may be returned when
    // a `Status` value received from another address space belongs to
    // an error space that is not known in this address space.  Also
    // errors raised by APIs that do not return enough error information
    // may be converted to this error.
    //
    // HTTP Mapping: 500 INTERNAL Server Error
    Unknown = 2,

    // The client specified an invalid argument.  Note that this differs
    // from `FAILED_PRECONDITION`.  `INVALID_ARGUMENT` indicates arguments
    // that are problematic regardless of the state of the system
    // (e.g., a malformed file name).
    //
    // HTTP Mapping: 400 Bad Request
    InvalidArgument = 3,

    // The deadline expired before the operation could complete. For operations
    // that change the state of the system, this error may be returned
    // even if the operation has completed successfully.  For example, a
    // successful response from a server could have been delayed long
    // enough for the deadline to expire.
    //
    // HTTP Mapping: 504 Gateway Timeout
    DeadlineExceeded = 4,

    // Some requested entity (e.g., file or directory) was not found.
    //
    // Note to server developers: if a request is denied for an entire class
    // of users, such as gradual feature rollout or undocumented whitelist,
    // `NOT_FOUND` may be used. If a request is denied for some users within
    // a class of users, such as user-based access control, `PERMISSION_DENIED`
    // must be used.
    //
    // HTTP Mapping: 404 Not Found
    NotFound = 5,

    // The entity that a client attempted to create (e.g., file or directory)
    // already exists.
    //
    // HTTP Mapping: 409 Conflict
    AlreadyExists = 6,

    // The caller does not have permission to execute the specified
    // operation. `PERMISSION_DENIED` must not be used for rejections
    // caused by exhausting some resource (use `RESOURCE_EXHAUSTED`
    // instead for those errors). `PERMISSION_DENIED` must not be
    // used if the caller can not be identified (use `UNAUTHENTICATED`
    // instead for those errors). This error code does not imply the
    // request is valid or the requested entity exists or satisfies
    // other pre-conditions.
    //
    // HTTP Mapping: 403 Forbidden
    PermissionDenied = 7,

    // Some resource has been exhausted, perhaps a per-user quota, or
    // perhaps the entire file system is out of space.
    //
    // HTTP Mapping: 429 Too Many Requests
    ResourceExhausted = 8,

    // The operation was rejected because the system is not in a state
    // required for the operation's execution.  For example, the directory
    // to be deleted is non-empty, an rmdir operation is applied to
    // a non-directory, etc.
    //
    // Service implementors can use the following guidelines to decide
    // between `FAILED_PRECONDITION`, `ABORTED`, and `UNAVAILABLE`:
    //  (a) Use `UNAVAILABLE` if the client can retry just the failing call.
    //  (b) Use `ABORTED` if the client should retry at a higher level
    //      (e.g., when a client-specified test-and-set fails, indicating the
    //      client should restart a read-modify-write sequence).
    //  (c) Use `FAILED_PRECONDITION` if the client should not retry until
    //      the system state has been explicitly fixed.  E.g., if an "rmdir"
    //      fails because the directory is non-empty, `FAILED_PRECONDITION`
    //      should be returned since the client should not retry unless
    //      the files are deleted from the directory.
    //
    // HTTP Mapping: 400 Bad Request
    FailedPrecondition = 9,

    // The operation was aborted, typically due to a concurrency issue such as
    // a sequencer check failure or transaction abort.
    //
    // See the guidelines above for deciding between `FAILED_PRECONDITION`,
    // `ABORTED`, and `UNAVAILABLE`.
    //
    // HTTP Mapping: 409 Conflict
    Aborted = 10,

    // The operation was attempted past the valid range.  E.g., seeking or
    // reading past end-of-file.
    //
    // Unlike `INVALID_ARGUMENT`, this error indicates a problem that may
    // be fixed if the system state changes. For example, a 32-bit file
    // system will generate `INVALID_ARGUMENT` if asked to read at an
    // offset that is not in the range [0,2^32-1], but it will generate
    // `OUT_OF_RANGE` if asked to read from an offset past the current
    // file size.
    //
    // There is a fair bit of overlap between `FAILED_PRECONDITION` and
    // `OUT_OF_RANGE`.  We recommend using `OUT_OF_RANGE` (the more specific
    // error) when it applies so that callers who are iterating through
    // a space can easily look for an `OUT_OF_RANGE` error to detect when
    // they are done.
    //
    // HTTP Mapping: 400 Bad Request
    OutOfRange = 11,

    // The operation is not implemented or is not supported/enabled in this
    // service.
    //
    // HTTP Mapping: 501 Not Implemented
    Unimplemented = 12,

    // INTERNAL errors.  This means that some invariants expected by the
    // underlying system have been broken.  This error code is reserved
    // for serious errors.
    //
    // HTTP Mapping: 500 INTERNAL Server Error
    Internal = 13,

    // The service is currently unavailable.  This is most likely a
    // transient condition, which can be corrected by retrying with
    // a backoff. Note that it is not always safe to retry
    // non-idempotent operations.
    //
    // See the guidelines above for deciding between `FAILED_PRECONDITION`,
    // `ABORTED`, and `UNAVAILABLE`.
    //
    // HTTP Mapping: 503 Service Unavailable
    Unavailable = 14,

    // Unrecoverable data loss or corruption.
    //
    // HTTP Mapping: 500 INTERNAL Server Error
    DataLoss = 15,

    // The request does not have valid authentication credentials for the
    // operation.
    //
    // HTTP Mapping: 401 Unauthorized
    Unauthenticated = 16,
}

impl Code {
    pub fn from_i32(i: i32) -> Code {
        Code::from(i)
    }

    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Unknown => "Unknown error",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            _ => "aa",
        }
    }

    pub fn to_http_header_value(&self) -> HeaderValue {
        match *self {
            Code::Ok => HeaderValue::from_static("0"),
            Code::Cancelled => HeaderValue::from_static("1"),
            Code::Unknown => HeaderValue::from_static("2"),
            Code::InvalidArgument => HeaderValue::from_static("3"),
            Code::DeadlineExceeded => HeaderValue::from_static("4"),
            Code::NotFound => HeaderValue::from_static("5"),
            Code::AlreadyExists => HeaderValue::from_static("6"),
            Code::PermissionDenied => HeaderValue::from_static("7"),
            Code::ResourceExhausted => HeaderValue::from_static("8"),
            Code::FailedPrecondition => HeaderValue::from_static("9"),
            Code::Aborted => HeaderValue::from_static("10"),
            Code::OutOfRange => HeaderValue::from_static("11"),
            Code::Unimplemented => HeaderValue::from_static("12"),
            Code::Internal => HeaderValue::from_static("13"),
            Code::Unavailable => HeaderValue::from_static("14"),
            Code::DataLoss => HeaderValue::from_static("15"),
            Code::Unauthenticated => HeaderValue::from_static("16"),
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.description(), f)
    }
}

impl From<i32> for Code {
    fn from(i: i32) -> Self {
        match i {
            0 => Code::Ok,
            1 => Code::Cancelled,
            2 => Code::Unknown,
            3 => Code::InvalidArgument,
            4 => Code::DeadlineExceeded,
            5 => Code::NotFound,
            6 => Code::AlreadyExists,
            7 => Code::PermissionDenied,
            8 => Code::ResourceExhausted,
            9 => Code::FailedPrecondition,
            10 => Code::Aborted,
            11 => Code::OutOfRange,
            12 => Code::Unimplemented,
            13 => Code::Internal,
            14 => Code::Unavailable,
            15 => Code::DataLoss,
            16 => Code::Unauthenticated,

            _ => Code::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    // grpc-status
    code: Code,

    // grpc-message
    message: String,
}

impl Status {
    pub fn new(code: Code, message: String) -> Self {
        Status { code, message }
    }

    pub fn with_message(self, message: String) -> Self {
        Status { message, ..self }
    }

    pub fn from_std_erro<T: std::error::Error>(err: T) -> Self {
        Status::new(Code::Internal, err.to_string())
    }

    pub fn from_error(err: crate::Error) -> Self {
        Status::new(Code::Internal, err.to_string())
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn to_http(&self) -> http::Response<crate::BoxBody> {
        let (mut parts, _) = http::Response::new(()).into_parts();

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );

        parts
            .headers
            .insert(GRPC_STATUS, self.code.to_http_header_value());
        parts.headers.insert(
            GRPC_MESSAGE,
            http::HeaderValue::from_str(&self.message).unwrap(),
        );

        parts.headers.insert(
            "grpc-accept-encoding",
            http::HeaderValue::from_static("gzip,identity"),
        );

        http::Response::from_parts(parts, crate::empty_body())
    }

    pub fn to_hyper_body(&self) -> http::Response<hyper::Body> {
        let (mut parts, _) = http::Response::new(()).into_parts();

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );

        parts
            .headers
            .insert(GRPC_STATUS, self.code.to_http_header_value());
        parts.headers.insert(
            GRPC_MESSAGE,
            http::HeaderValue::from_str(&self.message).unwrap(),
        );

        parts.headers.insert(
            "grpc-accept-encoding",
            http::HeaderValue::from_static("gzip,identity"),
        );

        http::Response::from_parts(parts, hyper::Body::empty())
    }
}

impl From<std::io::Error> for Status {
    fn from(err: std::io::Error) -> Self {
        Status::new(crate::status::Code::Internal, err.to_string())
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "grpc status, code: {}, message: {}",
            self.code, self.message
        ))
    }
}

impl Error for Status {}

#[derive(Debug)]
pub struct DubboError {
    message: String,
}

impl DubboError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for DubboError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dubbo internal Error: {}", self.message)
    }
}

impl Error for DubboError {}

unsafe impl Send for DubboError {}

unsafe impl Sync for DubboError {}
