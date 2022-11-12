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

use url::Url;

pub const DUBBO: &str = "dubbo";

pub const TRIPLE: &str = "tri";

pub const PROVIDER: &str = "provider";

pub const CONSUMER: &str = "consumer";

pub const APPLICATION_KEY: &str = "application";

pub const APPLICATION_VERSION_KEY: &str = "application.version";

pub const APPLICATION_PROTOCOL_KEY: &str = "application-protocol";

pub const METADATA_SERVICE_PORT_KEY: &str = "metadata-service-port";

pub const METADATA_SERVICE_PROTOCOL_KEY: &str = "metadata-service-protocol";

pub const LIVENESS_PROBE_KEY: &str = "liveness-probe";

pub const READINESS_PROBE_KEY: &str = "readiness-probe";

pub const STARTUP_PROBE: &str = "startup-probe";

pub const REMOTE_APPLICATION_KEY: &str = "remote.application";

pub const ENABLED_KEY: &str = "enabled";

pub const DISABLED_KEY: &str = "disabled";

pub const DUBBO_MIGRATION_KEY: &str = "dubbo.migration.file";


pub const ANY_VALUE: &str = "*";

/**
 * @since 2.7.8
 */
pub const COMMA_SEPARATOR_CHAR: char = ',';

pub const COMMA_SEPARATOR: &str = ",";

pub const DOT_SEPARATOR: &str = ".";


pub const PATH_SEPARATOR: &str = "/";

pub const PROTOCOL_SEPARATOR: &str = "://";


pub const REGISTRY_SEPARATOR: &str = "|";


pub const SEMICOLON_SEPARATOR: &str = ";";




pub const DEFAULT_DIRECTORY: &str = "dubbo";

pub const PROTOCOL_KEY: &str = "protocol";

pub const DEFAULT_PROTOCOL: &str = "dubbo";


pub const CORE_THREADS_KEY: &str = "corethreads";


pub const ALIVE_KEY: &str = "alive";

pub const DEFAULT_THREADPOOL: &str = "limited";

pub const DEFAULT_CLIENT_THREADPOOL: &str = "cached";

pub const IO_THREADS_KEY: &str = "iothreads";

pub const KEEP_ALIVE_KEY: &str = "keep.alive";

pub const DEFAULT_QUEUES: usize = 0;

pub const DEFAULT_ALIVE: usize = 60 * 1000;

pub const TIMEOUT_KEY: &str = "timeout";

pub const DEFAULT_TIMEOUT: usize = 1000;

pub const SESSION_KEY: &str = "session";

// used by invocation attachments to transfer timeout from Consumer to Provider.
// works as a replacement of TIMEOUT_KEY on wire, which seems to be totally useless in previous releases).
pub const TIMEOUT_ATTACHMENT_KEY: &str = "_TO";

pub const TIMEOUT_ATTACHMENT_KEY_LOWER: &str = "_to";

pub const TIME_COUNTDOWN_KEY: &str = "timeout-countdown";

pub const ENABLE_TIMEOUT_COUNTDOWN_KEY: &str = "enable-timeout-countdown";

pub const REMOVE_VALUE_PREFIX: &str = "-";

pub const PROPERTIES_CHAR_SEPARATOR: &str = "-";

pub const UNDERLINE_SEPARATOR: &str = "_";

pub const SEPARATOR_REGEX: &str = "_|-";

pub const GROUP_CHAR_SEPARATOR: &str = ":";

pub const HIDE_KEY_PREFIX: &str = ".";

pub const DOT_REGEX: &str = "\\.";

pub const DEFAULT_KEY_PREFIX: &str = "default.";

pub const DEFAULT_KEY: &str = "default";

pub const PREFERRED_KEY: &str = "preferred";

/**
 * Default timeout value in milliseconds for server shutdown
 */
pub const DEFAULT_SERVER_SHUTDOWN_TIMEOUT: usize = 10000;

pub const SIDE_KEY: &str = "side";

pub const PROVIDER_SIDE: &str = "provider";

pub const CONSUMER_SIDE: &str = "consumer";

pub const ANYHOST_KEY: &str = "anyhost";

pub const ANYHOST_VALUE: &str = "0.0.0.0";

pub const LOCALHOST_KEY: &str = "localhost";

pub const LOCALHOST_VALUE: &str = "127.0.0.1";

pub const METHODS_KEY: &str = "methods";

pub const METHOD_KEY: &str = "method";

pub const PID_KEY: &str = "pid";

pub const TIMESTAMP_KEY: &str = "timestamp";

pub const GROUP_KEY: &str = "group";

pub const PATH_KEY: &str = "path";

pub const INTERFACE_KEY: &str = "interface";

pub const FILE_KEY: &str = "file";

pub const FILTER_KEY: &str = "filter";

pub const DUMP_DIRECTORY: &str = "dump.directory";

pub const CLASSIFIER_KEY: &str = "classifier";

pub const VERSION_KEY: &str = "version";

pub const REVISION_KEY: &str = "revision";

pub const METADATA_KEY: &str = "metadata-type";

pub const REPORT_METADATA_KEY: &str = "report-metadata";

pub const REPORT_DEFINITION_KEY: &str = "report-definition";

pub const DEFAULT_METADATA_STORAGE_TYPE: &str = "local";

pub const REMOTE_METADATA_STORAGE_TYPE: &str = "remote";

pub const GENERIC_KEY: &str = "generic";

/**
 * package version in the manifest
 */
pub const RELEASE_KEY: &str = "release";

pub const PROTOBUF_MESSAGE_CLASS_NAME: &str = "com.google.protobuf.Message";

pub const MAX_PROXY_COUNT: usize = 65535;

pub const MONITOR_KEY: &str = "monitor";
pub const CLUSTER_KEY: &str = "cluster";
pub const USERNAME_KEY: &str = "username";
pub const PASSWORD_KEY: &str = "password";
pub const HOST_KEY: &str = "host";
pub const PORT_KEY: &str = "port";
pub const DUBBO_IP_TO_BIND: &str = "DUBBO_IP_TO_BIND";

/**
 * broadcast cluster.
 */
pub const BROADCAST_CLUSTER: &str = "broadcast";

/**
 * The property name for {@link NetworkInterface#getDisplayName() the name of network interface} that
 * the Dubbo application prefers
 *
 * @since 2.7.6
 */
pub const DUBBO_PREFERRED_NETWORK_INTERFACE: &str = "dubbo.network.interface.preferred";

pub const SHUTDOWN_WAIT_KEY: &str = "dubbo.service.shutdown.wait";
pub const DUBBO_PROTOCOL: &str = "dubbo";

pub const DUBBO_LABELS: &str = "dubbo.labels";
pub const DUBBO_ENV_KEYS: &str = "dubbo.env.keys";

pub const CONFIG_CONFIGFILE_KEY: &str = "config-file";
pub const CONFIG_ENABLE_KEY: &str = "highest-priority";
pub const CONFIG_NAMESPACE_KEY: &str = "namespace";
pub const CHECK_KEY: &str = "check";

pub const BACKLOG_KEY: &str = "backlog";

pub const MOCK_HEARTBEAT_EVENT: &str = "H";
pub const READONLY_EVENT: &str = "R";

pub const REFERENCE_FILTER_KEY: &str = "reference.filter";

pub const HEADER_FILTER_KEY: &str = "header.filter";

pub const INVOCATION_INTERCEPTOR_KEY: &str = "invocation.interceptor";

pub const INVOKER_LISTENER_KEY: &str = "invoker.listener";

pub const REGISTRY_PROTOCOL_LISTENER_KEY: &str = "registry.protocol.listener";

pub const DUBBO_VERSION_KEY: &str = "dubbo";

pub const TAG_KEY: &str = "dubbo.tag";

/**
 * To decide whether to make connection when the client is created
 */
pub const LAZY_CONNECT_KEY: &str = "lazy";

pub const STUB_EVENT_KEY: &str = "dubbo.stub.event";

pub const REFERENCE_INTERCEPTOR_KEY: &str = "reference.interceptor";

pub const SERVICE_FILTER_KEY: &str = "service.filter";

pub const EXPORTER_LISTENER_KEY: &str = "exporter.listener";

/**
 * After simplify the registry, should add some parameter individually for provider.
 *
 * @since 2.7.0
 */
pub const EXTRA_KEYS_KEY: &str = "extra-keys";


pub const GENERIC_SERIALIZATION_DEFAULT: &str = "true";


pub const GENERIC_SERIALIZATION_PROTOBUF: &str = "protobuf-json";

pub const GENERIC_WITH_CLZ_KEY: &str = "generic.include.class";

/**
 * Whether to cache locally, default is true
 */
pub const REGISTRY_LOCAL_FILE_CACHE_ENABLED: &str = "file.cache";


/**
 * The limit of callback service instances for one interface on every client
 */
pub const CALLBACK_INSTANCES_LIMIT_KEY: &str = "callbacks";

/**
 * The default limit number for callback service instances
 *
 * @see #CALLBACK_INSTANCES_LIMIT_KEY
 */
pub const DEFAULT_CALLBACK_INSTANCES: usize = 1;

pub const LOADBALANCE_KEY: &str = "loadbalance";

pub const DEFAULT_LOADBALANCE: &str = "random";

pub const RETRIES_KEY: &str = "retries";

pub const FORKS_KEY: &str = "forks";

pub const DEFAULT_RETRIES: usize = 2;

pub const DEFAULT_FAILBACK_TIMES: usize = 3;

pub const INTERFACES: &str = "interfaces";

pub const SSL_ENABLED_KEY: &str = "ssl-enabled";

pub const SERVICE_PATH_PREFIX: &str = "service.path.prefix";

pub const IPV6_KEY: &str = "ipv6";

/**
 * The parameter key for the class path of the ServiceNameMapping {@link Properties} file
 *
 * @since 2.7.8
 */
pub const SERVICE_NAME_MAPPING_PROPERTIES_FILE_KEY: &str = "service-name-mapping.properties-path";

/**
 * The default class path of the ServiceNameMapping {@link Properties} file
 *
 * @since 2.7.8
 */
pub const DEFAULT_SERVICE_NAME_MAPPING_PROPERTIES_PATH: &str = "META-INF/dubbo/service-name-mapping.properties";

pub const CLASS_DESERIALIZE_BLOCK_ALL: &str = "dubbo.security.serialize.blockAllClassExceptAllow";

pub const CLASS_DESERIALIZE_ALLOWED_LIST: &str = "dubbo.security.serialize.allowedClassList";

pub const CLASS_DESERIALIZE_BLOCKED_LIST: &str = "dubbo.security.serialize.blockedClassList";


pub const SERIALIZE_BLOCKED_LIST_FILE_PATH: &str = "security/serialize.blockedlist";

pub const QOS_LIVE_PROBE_EXTENSION: &str = "dubbo.application.liveness-probe";

pub const QOS_READY_PROBE_EXTENSION: &str = "dubbo.application.readiness-probe";

pub const QOS_STARTUP_PROBE_EXTENSION: &str = "dubbo.application.startup-probe";

pub const REGISTRY_DELAY_NOTIFICATION_KEY: &str = "delay-notification";

pub const CACHE_CLEAR_TASK_INTERVAL: &str = "dubbo.application.url.cache.task.interval";
pub const CACHE_CLEAR_WAITING_THRESHOLD: &str = "dubbo.application.url.cache.clear.waiting";

pub const CLUSTER_INTERCEPTOR_COMPATIBLE_KEY: &str = "dubbo.application.cluster.interceptor.compatible";

pub const UTF8ENCODE: &str = "UTF-8";


pub const DEFAULT_VERSION: &str = "0.0.0";

pub const CLASS_DESERIALIZE_OPEN_CHECK: &str = "dubbo.security.serialize.openCheckClass";

pub const ROUTER_KEY: &str = "router";

pub const EXPORT_ASYNC_KEY: &str = "export-async";

pub const REFER_ASYNC_KEY: &str = "refer-async";

pub const EXPORT_BACKGROUND_KEY: &str = "export-background";

pub const REFER_BACKGROUND_KEY: &str = "refer-background";

pub const EXPORT_THREAD_NUM_KEY: &str = "export-thread-num";

pub const REFER_THREAD_NUM_KEY: &str = "refer-thread-num";

pub const DEFAULT_EXPORT_THREAD_NUM: usize = 10;

pub const DEFAULT_REFER_THREAD_NUM: usize = 10;

pub const DEFAULT_DELAY_NOTIFICATION_TIME: usize = 5000;

pub const DEFAULT_DELAY_EXECUTE_TIMES: usize = 10;

/**
 * Url merge processor key
 */
pub const URL_MERGE_PROCESSOR_KEY: &str = "url-merge-processor";

/**
 * use native image to compile dubbo's identifier
 */
pub const NATIVE: &str = "native";

pub const DUBBO_MONITOR_ADDRESS: &str = "dubbo.monitor.address";

pub const SERVICE_NAME_MAPPING_KEY: &str = "service-name-mapping";

pub const SCOPE_MODEL: &str = "scopeModel";

pub const SERVICE_MODEL: &str = "serviceModel";

/**
 * The property name for {@link NetworkInterface#getDisplayName() the name of network interface} that
 * the Dubbo application will be ignored
 *
 * @since 2.7.6
 */
pub const DUBBO_NETWORK_IGNORED_INTERFACE: &str = "dubbo.network.interface.ignored";

pub const OS_NAME_KEY: &str = "os.name";

pub const OS_LINUX_PREFIX: &str = "linux";

pub const OS_WIN_PREFIX: &str = "win";

pub const RECONNECT_TASK_TRY_COUNT: &str = "dubbo.reconnect.reconnectTaskTryCount";

pub const DEFAULT_RECONNECT_TASK_TRY_COUNT: usize = 10;

pub const RECONNECT_TASK_PERIOD: &str = "dubbo.reconnect.reconnectTaskPeriod";

pub const DEFAULT_RECONNECT_TASK_PERIOD: usize = 1000;

pub const RESELECT_COUNT: &str = "dubbo.reselect.count";

pub const DEFAULT_RESELECT_COUNT: usize = 10;

pub const ENABLE_CONNECTIVITY_VALIDATION: &str = "dubbo.connectivity.validation";

pub const DUBBO_INTERNAL_APPLICATION: &str = "DUBBO_INTERNAL_APPLICATION";

pub const RETRY_TIMES_KEY: &str = "retry-times";

pub const RETRY_PERIOD_KEY: &str = "retry-period";

pub const SYNC_REPORT_KEY: &str = "sync-report";

pub const CYCLE_REPORT_KEY: &str = "cycle-report";

pub const WORKING_CLASSLOADER_KEY: &str = "WORKING_CLASSLOADER";

pub const STAGED_CLASSLOADER_KEY: &str = "STAGED_CLASSLOADER";

pub const PROVIDER_ASYNC_KEY: &str = "PROVIDER_ASYNC";

pub const REGISTER_IP_KEY: &str = "register.ip";

pub const CURRENT_CLUSTER_INVOKER_KEY: &str = "currentClusterInvoker";

pub const ENABLE_ROUTER_SNAPSHOT_PRINT_KEY: &str = "ENABLE_ROUTER_SNAPSHOT_PRINT";


pub const SET_FUTURE_IN_SYNC_MODE: &str = "future.sync.set";

pub const CLEAR_FUTURE_AFTER_GET: &str = "future.clear.once";

pub const NATIVE_STUB: &str = "nativestub";

pub const METADATA: &str = "metadata";

pub const IGNORE_LISTEN_SHUTDOWN_HOOK: &str = "dubbo.shutdownHook.listenIgnore";

pub const OPTIMIZER_KEY: &str = "optimizer";

pub const PREFER_JSON_FRAMEWORK_NAME: &str = "dubbo.json-framework.prefer";

/**
 * @since 3.1.0
 */
pub const MESH_ENABLE: &str = "mesh-enable";

/**
 * @since 3.1.0
 */
pub const DEFAULT_MESH_PORT: usize = 80;

/**
 * @since 3.1.0
 */
pub const SVC: &str = ".svc.";

/**
 * Domain name suffix used inside k8s.
 *
 * @since 3.1.0
 */
pub const DEFAULT_CLUSTER_DOMAIN: &str = "cluster.local";

/**
 * @since 3.1.0
 */
pub const UNLOAD_CLUSTER_RELATED: &str = "unloadClusterRelated";

