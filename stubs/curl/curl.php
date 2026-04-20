<?php

/**
 * Initialize a cURL session.
 *
 * @php-since 7.4
 */
function curl_init(?string $url = null): CurlHandle|false {}

/**
 * Set an option for a cURL transfer.
 *
 * @php-since 7.4
 */
function curl_setopt(CurlHandle $handle, int $option, mixed $value): bool {}

/**
 * Perform a cURL session.
 *
 * @php-since 7.4
 */
function curl_exec(CurlHandle $handle): string|bool {}

/**
 * Close a cURL session.
 *
 * @deprecated 8.0 Use unset() instead.
 * @php-since 7.4
 */
function curl_close(CurlHandle $handle): void {}

/**
 * Return a string containing the last error for the current session.
 *
 * @php-since 7.4
 */
function curl_error(CurlHandle $handle): string {}

/**
 * Return the last error number.
 *
 * @php-since 7.4
 */
function curl_errno(CurlHandle $handle): int {}

/**
 * Get information regarding a specific transfer.
 *
 * @php-since 7.4
 */
function curl_getinfo(CurlHandle $handle, ?int $option = null): mixed {}

/**
 * Set multiple options for a cURL transfer.
 *
 * @php-since 7.4
 */
function curl_setopt_array(CurlHandle $handle, array $options): bool {}

/**
 * Returns a new cURL multi handle.
 *
 * @php-since 7.4
 */
function curl_multi_init(): CurlMultiHandle {}

/**
 * Add a normal cURL handle to a cURL multi handle.
 *
 * @php-since 7.4
 */
function curl_multi_add_handle(CurlMultiHandle $multi_handle, CurlHandle $handle): int {}

/**
 * Run the sub-connections of the current cURL handle.
 *
 * @param int $still_running Set to the number of transfers still in progress.
 * @php-since 7.4
 */
function curl_multi_exec(CurlMultiHandle $multi_handle, int &$still_running): int {}

/**
 * Return the content of a cURL handle if CURLOPT_RETURNTRANSFER is set.
 *
 * @php-since 7.4
 */
function curl_multi_getcontent(CurlHandle $handle): ?string {}

/**
 * Remove a multi handle from a set of cURL handles.
 *
 * @php-since 7.4
 */
function curl_multi_remove_handle(CurlMultiHandle $multi_handle, CurlHandle $handle): int {}

/**
 * Close a set of cURL handles.
 *
 * @php-since 7.4
 */
function curl_multi_close(CurlMultiHandle $multi_handle): void {}

/**
 * Reset all options set on the given cURL handle to the default values.
 *
 * @php-since 7.4
 */
function curl_reset(CurlHandle $handle): void {}
