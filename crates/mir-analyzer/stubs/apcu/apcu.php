<?php

/**
 * Cache a variable in the data store.
 *
 * @param string $key
 * @param mixed  $var
 * @param int    $ttl Time-to-live in seconds (0 = never expire).
 */
function apcu_store(string $key, mixed $var, int $ttl = 0): bool {}

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
 * @param string|string[] $key
 * @return bool|string[] Returns true on success, false on failure. If an array
 *                       of keys is passed, returns an array of failed keys.
 */
function apcu_delete(string|array $key): bool|array {}

/**
 * Checks if one or more APCu keys exist.
 *
 * @param string|string[] $keys
 * @return bool|string[] Returns true/false for a single key, or an array of
 *                       existing keys when an array is passed.
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
 * @param string $key
 * @param mixed  $var
 * @param int    $ttl Time-to-live in seconds (0 = never expire).
 */
function apcu_add(string $key, mixed $var, int $ttl = 0): bool {}

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
