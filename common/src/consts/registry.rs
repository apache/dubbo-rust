pub const REGISTRY_KEY: &str = "registry";

pub const REGISTRY_CLUSTER_KEY: &str = "REGISTRY_CLUSTER";

pub const REGISTRY_CLUSTER_TYPE_KEY: &str = "registry-cluster-type";

pub const REGISTRY_PROTOCOL: &str = "registry";

pub const DYNAMIC_KEY: &str = "dynamic";

pub const CATEGORY_KEY: &str = "category";

pub const PROVIDERS_CATEGORY: &str = "providers";

pub const CONSUMERS_CATEGORY: &str = "consumers";

pub const ROUTERS_CATEGORY: &str = "routers";

pub const DYNAMIC_ROUTERS_CATEGORY: &str = "dynamicrouters";

pub const DEFAULT_CATEGORY: &str = PROVIDERS_CATEGORY;

pub const CONFIGURATORS_CATEGORY: &str = "configurators";

pub const ALL_CATEGORIES: &str = "providers,configurators,routers";

pub const DYNAMIC_CONFIGURATORS_CATEGORY: &str = "dynamicconfigurators";

pub const APP_DYNAMIC_CONFIGURATORS_CATEGORY: &str = "appdynamicconfigurators";

pub const ROUTERS_SUFFIX: &str = ".routers";

pub const EMPTY_PROTOCOL: &str = "empty";

pub const ROUTE_PROTOCOL: &str = "route";

pub const ROUTE_SCRIPT_PROTOCOL: &str = "script";

pub const OVERRIDE_PROTOCOL: &str = "override";

pub const COMPATIBLE_CONFIG_KEY: &str = "compatible_config";

pub const REGISTER_MODE_KEY: &str = "register-mode";

pub const DUBBO_REGISTER_MODE_DEFAULT_KEY: &str = "dubbo.application.register-mode";

pub const DUBBO_PUBLISH_INTERFACE_DEFAULT_KEY: &str = "dubbo.application.publish-interface";

pub const DUBBO_PUBLISH_INSTANCE_DEFAULT_KEY: &str = "dubbo.application.publish-instance";

pub const DEFAULT_REGISTER_MODE_INTERFACE: &str = "interface";

pub const DEFAULT_REGISTER_MODE_INSTANCE: &str = "instance";

pub const DEFAULT_REGISTER_MODE_ALL: &str = "all";
/**
 * The parameter key of Dubbo Registry type
 *
 * @since 2.7.5
 */
pub const REGISTRY_TYPE_KEY: &str = "registry-type";

/**
 * The parameter value of Service-Oriented Registry type
 *
 * @since 2.7.5
 */
pub const SERVICE_REGISTRY_TYPE: &str = "service";

/**
 * The protocol for Service Discovery
 *
 * @since 2.7.5
 */
pub const SERVICE_REGISTRY_PROTOCOL: &str = "service-discovery-registry";

/**
 * Specify registry level services consumer needs to subscribe to, multiple values should be separated using ",".
 */
pub const SUBSCRIBED_SERVICE_NAMES_KEY: &str = "subscribed-services";

pub const PROVIDED_BY: &str = "provided-by";

/**
 * The provider tri port
 *
 * @since 3.1.0
 */
pub const PROVIDER_PORT: &str = "provider-port";

/**
 * provider namespace
 *
 * @since 3.1.1
 */
pub const PROVIDER_NAMESPACE: &str = "provider-namespace";

/**
 * The request size of service instances
 *
 * @since 2.7.5
 */
pub const INSTANCES_REQUEST_SIZE_KEY: &str = "instances-request-size";

/**
 * The default request size of service instances
 */
pub const DEFAULT_INSTANCES_REQUEST_SIZE: usize = 100;

pub const ACCEPTS_KEY: &str = "accepts";

pub const REGISTRY_ZONE: &str = "registry_zone";
pub const REGISTRY_ZONE_FORCE: &str = "registry_zone_force";
pub const ZONE_KEY: &str = "zone";

pub const INIT: &str = "INIT";

pub const DEFAULT_HASHMAP_LOAD_FACTOR: f32 = 0.75;

pub const ENABLE_EMPTY_PROTECTION_KEY: &str = "enable-empty-protection";
pub const REGISTER_CONSUMER_URL_KEY: &str = "register-consumer-url";