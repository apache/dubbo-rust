use std::{marker::PhantomData, sync::Arc};

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

// inner: Box<dyn Service<T, Response = U, Error = E, Future = Pin<Box<dyn Future<Output = Result<U, E>> + Send>>> + Send>,
pub struct BoxedService<N, R> {
    inner: N,
    _mark: PhantomData<R>,
}

impl<R, N> BoxedService<N, R> {
    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner: N| Self {
            inner,
            _mark: PhantomData,
        })
    }
}

// impl<T, R, N> NewService<T> for BoxedService<N, R>
// where
//     N: NewService<T>,
//     N::Service: Service<R> + Send,
//     <N::Service as Service<R>>::Future: Send + 'static,
// {
//     type Service = Box<dyn Service<R, Response = <N::Service as Service<R>>::Response, Error = <N::Service as Service<R>>::Error, Future = Pin<Box<dyn Future<Output = Result<<N::Service as Service<R>>::Response, <N::Service as Service<R>>::Error>> + Send>>> + Send>;

//     fn new_service(&self, target: T) -> Self::Service {
//         let service = self.inner.new_service(target);
//         Box::new(service.map_future(|f|Box::pin(f) as _))
//     }
// }
