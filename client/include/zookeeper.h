#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define ZOO_PERM_READ (1 << 0)

#define ZOO_PERM_WRITE (1 << 1)

#define ZOO_PERM_CREATE (1 << 2)

#define ZOO_PERM_DELETE (1 << 3)

#define ZOO_PERM_ADMIN (1 << 4)

#define ZOO_PERM_ALL 31

#define ZOO_READONLY 1

#define ZOO_NO_LOG_CLIENTENV 2

#define ZOOKEEPER_WRITE (1 << 0)

#define ZOOKEEPER_READ (1 << 1)

#define ZOO_PERSISTENT 0

#define ZOO_EPHEMERAL 1

#define ZOO_PERSISTENT_SEQUENTIAL 2

#define ZOO_EPHEMERAL_SEQUENTIAL 3

#define ZOO_CONTAINER 4

#define ZOO_PERSISTENT_WITH_TTL 5

#define ZOO_PERSISTENT_SEQUENTIAL_WITH_TTL 6

#define ZOO_SEQUENCE (1 << 1)

#define ZOO_EXPIRED_SESSION_STATE -112

#define ZOO_AUTH_FAILED_STATE -113

#define ZOO_CONNECTING_STATE 1

#define ZOO_ASSOCIATING_STATE 2

#define ZOO_CONNECTED_STATE 3

#define ZOO_READONLY_STATE 5

#define ZOO_SSL_CONNECTING_STATE 7

#define ZOO_NOTCONNECTED_STATE 999

#define ZOO_CREATED_EVENT 1

#define ZOO_DELETED_EVENT 2

#define ZOO_CHANGED_EVENT 3

#define ZOO_CHILD_EVENT 4

#define ZOO_SESSION_EVENT -1

#define ZOO_NOTWATCHING_EVENT -2

typedef struct Id Id;

typedef struct clientid_t clientid_t;

typedef struct zhandle_t zhandle_t;

typedef int32_t (*watcher_fn)(struct zhandle_t *zhandle_t,
                              int32_t type,
                              int32_t state,
                              const char *path,
                              void *watcherCtx);

extern const CStr *ZOO_CONFIG_NODE;

extern const struct Id *ZOO_ANYONE_ID_UNSAFE;

extern const struct Id *ZOO_AUTH_IDS;

const struct zhandle_t *zookeeper_init(const char *host,
                                       watcher_fn watcher,
                                       int32_t recv_timeout,
                                       const struct clientid_t *client_id,
                                       void *context,
                                       int32_t flags);

void zookeeper_close(const struct zhandle_t *zh);

int32_t zoo_get(const struct zhandle_t *zh,
                const char *path,
                int32_t watch,
                char *buffer,
                int *buffer_len,
                const Stat *stat);
