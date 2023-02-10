# Introduction

The goal of this package is to complete the separation of the abstraction and underlying packaging from the concrete implementation, with a possible directory structure as follows:

/dubbo-rust
    /dubbo-core
        /src 
            /lifecycle # Dubbo lifecycle abstraction and necessity implementations (eg. provider/consumer service builder)
            /layer_abstract # config/cluster/filter/registry/context and other abstraction
            /ext # abstraction of extension points
            /net # eg. network layer tcp stream->bytes,approximately equals to hyper
            /macros # shared macros
            /utils

## dubbo-core package focuses

1. abstract design of Dubbo lifecycle management and the necessary implementation (based on tower)
2. provide the abstract design of the cluster/registry/filter/protocol layer
3. define the necessary constructs and traits required by Dubbo (no implementation included)
4. provide the underlying library or system access layer packaging, such as network communication, file reading and some tool classes, etc.
5. provide the abstract design of user extension points

## differences with dubbo package

1. initialize the Dubbo instance and load the user configuration based on the abstraction provided by dubbo-core
2. provide concrete implementation of config/cluster/registry/filter/protocol layers
3. load the concrete implementation of user extension points
3. load the concrete implementation based on the user configuration during the Dubbo lifecycle