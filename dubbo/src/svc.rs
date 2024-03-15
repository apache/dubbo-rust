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
use std::sync::Arc;

pub trait NewService<T> {
    type Service;

    fn new_service(&self, target: T) -> Self::Service;
}

pub struct ArcNewService<T, S> {
    inner: Arc<dyn NewService<T, Service = S> + Send + Sync>,
}

impl<T, S> ArcNewService<T, S> {
    pub fn layer<N>() -> impl tower_layer::Layer<N, Service = Self> + Clone + Copy
    where
        N: NewService<T, Service = S> + Send + Sync + 'static,
        S: Send + 'static,
    {
        tower_layer::layer_fn(Self::new)
    }

    pub fn new<N>(inner: N) -> Self
    where
        N: NewService<T, Service = S> + Send + Sync + 'static,
        S: Send + 'static,
    {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<T, S> Clone for ArcNewService<T, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, S> NewService<T> for ArcNewService<T, S> {
    type Service = S;

    fn new_service(&self, t: T) -> S {
        self.inner.new_service(t)
    }
}
