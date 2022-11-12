pub const FAIL_BACK_TASKS_KEY: &str = "failbacktasks";

pub const DEFAULT_FAILBACK_TASKS: usize = 100;

pub const DEFAULT_FORKS: usize = 2;

pub const WEIGHT_KEY: &str = "weight";

pub const DEFAULT_WEIGHT: usize = 100;

pub const MOCK_PROTOCOL: &str = "mock";

pub const FORCE_KEY: &str = "force";

pub const RAW_RULE_KEY: &str = "rawRule";

pub const VALID_KEY: &str = "valid";

pub const ENABLED_KEY: &str = "enabled";

pub const DYNAMIC_KEY: &str = "dynamic";

pub const SCOPE_KEY: &str = "scope";

pub const KEY_KEY: &str = "key";

pub const CONDITIONS_KEY: &str = "conditions";

pub const TAGS_KEY: &str = "tags";

/**
 * To decide whether to exclude unavailable invoker from the cluster
 */
pub const CLUSTER_AVAILABLE_CHECK_KEY: &str = "cluster.availablecheck";

/**
 * The default value of cluster.availablecheck
 *
 * @see #CLUSTER_AVAILABLE_CHECK_KEY
 */
pub const DEFAULT_CLUSTER_AVAILABLE_CHECK: bool = true;

/**
 * To decide whether to enable sticky strategy for cluster
 */
pub const CLUSTER_STICKY_KEY: &str = "sticky";

/**
 * The default value of sticky
 *
 * @see #CLUSTER_STICKY_KEY
 */
pub const DEFAULT_CLUSTER_STICKY: bool = false;

pub const ADDRESS_KEY: &str = "address";

/**
 * When this attribute appears in invocation's attachment, mock invoker will be used
 */
pub const INVOCATION_NEED_MOCK: &str = "invocation.need.mock";

/**
 * when ROUTER_KEY's value is set to ROUTER_TYPE_CLEAR, RegistryDirectory will clean all current routers
 */
pub const ROUTER_TYPE_CLEAR: &str = "clean";

pub const DEFAULT_SCRIPT_TYPE_KEY: &str = "javascript";

pub const PRIORITY_KEY: &str = "priority";

pub const RULE_KEY: &str = "rule";

pub const TYPE_KEY: &str = "type";

pub const RUNTIME_KEY: &str = "runtime";

pub const WARMUP_KEY: &str = "warmup";

pub const DEFAULT_WARMUP: usize = 10 * 60 * 1000;

pub const CONFIG_VERSION_KEY: &str = "configVersion";

pub const OVERRIDE_PROVIDERS_KEY: &str = "providerAddresses";


/**
 * key for router type, for e.g., "script"/"file",  corresponding to ScriptRouterFactory.NAME, FileRouterFactory.NAME
 */
pub const ROUTER_KEY: &str = "router";

/**
 * The key name for reference URL in register center
 */
pub const REFER_KEY: &str = "refer";

pub const ATTRIBUTE_KEY: &str = "attribute";

/**
 * The key name for export URL in register center
 */
pub const EXPORT_KEY: &str = "export";

pub const PEER_KEY: &str = "peer";

pub const CONSUMER_URL_KEY: &str = "CONSUMER_URL";

/**
 * prefix of arguments router key
 */
pub const ARGUMENTS: &str = "arguments";

pub const NEED_REEXPORT: &str = "need-reexport";

/**
 * The key of shortestResponseSlidePeriod
 */
pub const SHORTEST_RESPONSE_SLIDE_PERIOD: &str = "shortestResponseSlidePeriod";

pub const SHOULD_FAIL_FAST_KEY: &str = "dubbo.router.should-fail-fast";