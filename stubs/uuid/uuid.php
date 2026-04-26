<?php

const UUID_TYPE_DEFAULT = 0;
const UUID_TYPE_TIME = 1;
const UUID_TYPE_DCE = 2;
const UUID_TYPE_NAME_MD5 = 3;
const UUID_TYPE_RANDOM = 4;
const UUID_TYPE_NAME_SHA1 = 5;
const UUID_TYPE_NULL = -1;
const UUID_TYPE_INVALID = -42;

/**
 * Generate a new UUID.
 *
 * @param int $uuid_type UUID version to generate (one of the UUID_TYPE_* constants).
 */
function uuid_create(int $uuid_type = UUID_TYPE_DEFAULT): string {}

/**
 * Check whether a UUID is valid.
 *
 * @param string $uuid The UUID string to validate.
 */
function uuid_is_valid(string $uuid): bool {}

/**
 * Compare two UUIDs.
 *
 * @param string $uuid1
 * @param string $uuid2
 * @return int Returns -1, 0, or 1 if uuid1 is less than, equal to, or greater than uuid2.
 */
function uuid_compare(string $uuid1, string $uuid2): int {}

/**
 * Check whether a UUID is the null UUID.
 *
 * @param string $uuid
 */
function uuid_is_null(string $uuid): bool {}

/**
 * Return the UUID type.
 *
 * @param string $uuid
 * @return int One of the UUID_TYPE_* constants.
 */
function uuid_type(string $uuid): int {}

/**
 * Return the UUID variant.
 *
 * @param string $uuid
 */
function uuid_variant(string $uuid): int {}

/**
 * Extract the time from a time-based UUID.
 *
 * @param string $uuid A UUID_TYPE_TIME UUID.
 * @return int Unix timestamp encoded in the UUID.
 */
function uuid_time(string $uuid): int {}

/**
 * Extract the MAC address from a time-based UUID.
 *
 * @param string $uuid A UUID_TYPE_TIME UUID.
 */
function uuid_mac(string $uuid): string {}

/**
 * Convert a UUID string to a binary representation.
 *
 * @param string $uuid
 */
function uuid_parse(string $uuid): string {}

/**
 * Convert a binary UUID to its string representation.
 *
 * @param string $uuid Binary UUID (16 bytes).
 */
function uuid_unparse(string $uuid): string {}
