<?php

/**
 * Terminate apache process after this request.
 * @link https://php.net/manual/en/function.apache-child-terminate.php
 */
function apache_child_terminate(): void {}

/**
 * Fetch Apache's module list.
 * @link https://php.net/manual/en/function.apache-get-modules.php
 * @return array An array of loaded Apache modules.
 */
function apache_get_modules(): array {}

/**
 * Fetch Apache version.
 * @link https://php.net/manual/en/function.apache-get-version.php
 * @return string|false The Apache version string, or false on failure.
 */
function apache_get_version(): string|false {}

/**
 * Get an Apache subprocess_env variable.
 * @link https://php.net/manual/en/function.apache-getenv.php
 * @param string $variable The environment variable name.
 * @param bool   $walk_to_top Whether to get the top-level variable available to all Apache layers.
 * @return string|false The value of the variable, or false if not found.
 */
function apache_getenv(string $variable, bool $walk_to_top = false): string|false {}

/**
 * Perform a partial request for the specified URI and return all info about it.
 * @link https://php.net/manual/en/function.apache-lookup-uri.php
 * @param string $filename The URI to look up.
 * @return object|false An object with properties describing the request, or false on failure.
 */
function apache_lookup_uri(string $filename): object|false {}

/**
 * Get/set an Apache request note.
 * @link https://php.net/manual/en/function.apache-note.php
 * @param string $note_name The note name.
 * @param string $note_value When present, sets the note to this value.
 * @return string The prior value of the note.
 */
function apache_note(string $note_name, string $note_value = ""): string {}

/**
 * Reset the Apache write timer for the current request.
 * @link https://php.net/manual/en/function.apache-reset-timeout.php
 */
function apache_reset_timeout(): true {}

/**
 * Fetch all HTTP response headers.
 * @link https://php.net/manual/en/function.apache-response-headers.php
 * @return array An associative array of all the response headers.
 */
function apache_response_headers(): array {}

/**
 * Set an Apache subprocess_env variable.
 * @link https://php.net/manual/en/function.apache-setenv.php
 * @param string $variable The environment variable name.
 * @param string $value The value to set.
 * @param bool   $walk_to_top Whether to set the variable at the top-level Apache layer too.
 */
function apache_setenv(string $variable, string $value, bool $walk_to_top = false): true {}

/**
 * Perform an Apache sub-request for the specified URI and include its output.
 * @link https://php.net/manual/en/function.virtual.php
 * @param string $filename A path relative to the DocumentRoot for the current URI.
 * @return bool True on success. Throws an E_ERROR if the sub-request fails.
 */
function virtual(string $filename): bool {}
