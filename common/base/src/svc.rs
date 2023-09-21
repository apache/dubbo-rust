pub trait NewService<T> {
    
    type Service;

    fn new_service(&self, target: T) -> Self::Service;

}