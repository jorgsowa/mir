<?php

const APC_LIST_ACTIVE = 1;
const APC_LIST_DELETED = 2;
const APC_ITER_TYPE = 1;
const APC_ITER_KEY = 2;
const APC_ITER_FILENAME = 4;
const APC_ITER_DEVICE = 8;
const APC_ITER_INODE = 16;
const APC_ITER_VALUE = 32;
const APC_ITER_MD5 = 64;
const APC_ITER_NUM_HITS = 128;
const APC_ITER_MTIME = 256;
const APC_ITER_CTIME = 512;
const APC_ITER_DTIME = 1024;
const APC_ITER_ATIME = 2048;
const APC_ITER_REFCOUNT = 4096;
const APC_ITER_MEM_SIZE = 8192;
const APC_ITER_TTL = 16384;
const APC_ITER_NONE = 0;
const APC_ITER_ALL = -1;

/**
 * Cache a variable in the data store.
 *
 * @param string|array<string, mixed> $key A key, or an array of key => value pairs.
 * @param mixed                       $var Ignored when $key is an array.
 * @param int                         $ttl Time-to-live in seconds (0 = never expire).
 * @return ($key is array ? array<string, int>|false : bool) An array of the keys that failed when $key is an array.
 */
function apcu_store(string|array $key, mixed $var = null, int $ttl = 0): bool|array {}

/**
 * Fetch a stored variable from the cache.
 *
 * @param string|string[] $key
 * @param bool            $success Set to true on success, false on failure.
 * @return mixed Returns the stored variable or false on failure.
 */
function apcu_fetch(string|array $key, bool &$success = null): mixed {}

/**
 * Removes a stored variable from the cache.
 *
 * @param string|string[]|APCUIterator $key
 * @return ($key is array ? list<string> : bool) The keys that failed when $key is an array.
 */
function apcu_delete(string|array|APCUIterator $key): bool|array {}

/**
 * Checks if one or more APCu keys exist.
 *
 * @param string|string[] $keys
 * @return ($keys is array ? array<string, true> : bool) The existing keys when $keys is an array.
 */
function apcu_exists(string|array $keys): bool|array {}

/**
 * Clears the APCu cache.
 */
function apcu_clear_cache(): bool {}

/**
 * Increase a stored number.
 *
 * @param string $key
 * @param int    $step    Amount to increment by.
 * @param bool   $success Set to true on success, false on failure.
 * @param int    $ttl     Time-to-live in seconds (0 = never expire).
 * @return int|false Returns the new value or false on failure.
 */
function apcu_inc(string $key, int $step = 1, bool &$success = null, int $ttl = 0): int|false {}

/**
 * Decrease a stored number.
 *
 * @param string $key
 * @param int    $step    Amount to decrement by.
 * @param bool   $success Set to true on success, false on failure.
 * @param int    $ttl     Time-to-live in seconds (0 = never expire).
 * @return int|false Returns the new value or false on failure.
 */
function apcu_dec(string $key, int $step = 1, bool &$success = null, int $ttl = 0): int|false {}

/**
 * Cache a new variable in the data store (fails silently if key exists).
 *
 * @param string|array<string, mixed> $key A key, or an array of key => value pairs.
 * @param mixed                       $var Ignored when $key is an array.
 * @param int                         $ttl Time-to-live in seconds (0 = never expire).
 * @return ($key is array ? array<string, int>|false : bool) An array of the keys that failed when $key is an array.
 */
function apcu_add(string|array $key, mixed $var = null, int $ttl = 0): bool|array {}

/**
 * Retrieves cached information from APCu's data store.
 *
 * @param bool $limited When true, omits individual cache entries.
 * @return array|false Cache info array or false on failure.
 */
function apcu_cache_info(bool $limited = false): array|false {}

/**
 * Retrieves APCu shared memory allocation information.
 *
 * @param bool $limited When true, returns abbreviated info.
 * @return array|false SMA info array or false on failure.
 */
function apcu_sma_info(bool $limited = false): array|false {}

/**
 * Update an existing integer value if it still equals the old value.
 *
 * @param string $key
 * @param int    $old The value currently stored.
 * @param int    $new The value to update to.
 */
function apcu_cas(string $key, int $old, int $new): bool {}

/**
 * Atomically fetch a cache entry, or generate and store it via the callback.
 *
 * @template T
 * @param string            $key
 * @param callable(string):T $generator Receives the key and returns the value to cache.
 * @param int               $ttl       Time-to-live in seconds (0 = never expire).
 * @return T The cached (or freshly generated) value.
 */
function apcu_entry(string $key, callable $generator, int $ttl = 0): mixed {}

/**
 * Whether APCu is usable in the current environment.
 */
function apcu_enabled(): bool {}

/**
 * Get detailed information about a cache key.
 *
 * @param string $key
 * @return array|null Key information, or null if the key does not exist.
 */
function apcu_key_info(string $key): ?array {}

/**
 * Iterates over the entries of the APCu cache.
 *
 * @link https://php.net/manual/en/class.apcuiterator.php
 * @implements Iterator<string, array>
 */
class APCUIterator implements Iterator
{
    /**
     * @param string|string[]|null $search PCRE pattern(s) matched against key names, or null for all keys.
     * @param int                  $format One or more of the APC_ITER_* constants.
     * @param int                  $chunk_size Must be greater than 0.
     * @param int                  $list APC_LIST_ACTIVE or APC_LIST_DELETED.
     */
    public function __construct(array|string|null $search = null, int $format = APC_ITER_ALL, int $chunk_size = 100, int $list = APC_LIST_ACTIVE) {}

    public function rewind(): void {}

    public function valid(): bool {}

    public function current(): mixed {}

    public function key(): string|int {}

    public function next(): void {}

    public function getTotalHits(): int {}

    public function getTotalSize(): int {}

    public function getTotalCount(): int {}
}
