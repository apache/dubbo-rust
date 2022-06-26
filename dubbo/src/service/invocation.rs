use tonic::metadata::MetadataMap;

pub struct Request<T> {
    pub message: T,
    pub metadata: MetadataMap,
}


impl<T> Request<T> {
    pub fn new(message: T) -> Request<T> {
        Self {
            message,
            metadata: MetadataMap::new()
        }
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }
}

pub struct Response<T> {
    message: T,
    metadata: MetadataMap,
}

impl<T> Response<T> {
    pub fn new(message: T) -> Response<T> {
        Self {
            message,
            metadata: MetadataMap::new(),
        }
    }

    pub fn from_parts(metadata: MetadataMap, message: T) -> Self {
        Self {
            message,
            metadata,
        }
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }
}