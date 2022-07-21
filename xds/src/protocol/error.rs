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

use std::{convert::From, error, fmt, result, str};

pub type Result<T> = result::Result<T, Error>;

pub struct Error {
    repr: Repr,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

enum Repr {
    Other(String),
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    Protocol,
    IO,
    Client,
    Network,
    Server,
    Serialization,
    Other,
}

impl ErrorKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Protocol => "invalid protocol",
            ErrorKind::IO => "io issue",
            ErrorKind::Client => "client error",
            ErrorKind::Network => "network issue",
            ErrorKind::Server => "server error",
            ErrorKind::Serialization => "serialization failure",
            ErrorKind::Other => "other",
        }
    }
}

impl From<&'static str> for Error {
    #[inline]
    fn from(s: &'static str) -> Error {
        Error {
            repr: Repr::Other(s.to_owned()),
        }
    }
}

impl From<String> for Error {
    #[inline]
    fn from(s: String) -> Error {
        Error {
            repr: Repr::Other(s),
        }
    }
}

impl From<serde_json::error::Error> for Error {
    #[inline]
    fn from(err: serde_json::error::Error) -> Error {
        Error::new(ErrorKind::IO, err)
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(err: std::io::Error) -> Error {
        Error::new(ErrorKind::IO, err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    #[inline]
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Error {
        Error::_new(ErrorKind::Other, err)
    }
}
impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Error {
        Error {
            repr: Repr::Custom(Box::new(Custom { kind, error })),
        }
    }

    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match self.repr {
            Repr::Other(..) => None,
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => Some(&*c.error),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
        match self.repr {
            Repr::Other(..) => None,
            Repr::Simple(..) => None,
            Repr::Custom(ref mut c) => Some(&mut *c.error),
        }
    }

    pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
        match self.repr {
            Repr::Other(..) => None,
            Repr::Simple(..) => None,
            Repr::Custom(c) => Some(c.error),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Other(_) => ErrorKind::Other,
            Repr::Custom(ref c) => c.kind,
            Repr::Simple(kind) => kind,
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Repr::Other(s) => write!(fmt, "{}", s),
            Repr::Custom(ref c) => fmt::Debug::fmt(&c, fmt),
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Other(s) => write!(fmt, "{}", s),
            Repr::Custom(ref c) => c.error.fmt(fmt),
            Repr::Simple(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match &self.repr {
            Repr::Other(s) => &s[..],
            Repr::Simple(..) => self.kind().as_str(),
            Repr::Custom(ref c) => c.error.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self.repr {
            Repr::Other(..) => None,
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Other(..) => None,
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}
