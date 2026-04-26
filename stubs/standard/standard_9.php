<?php

use JetBrains\PhpStorm\Pure;

/**
 * Return all the keys or a subset of the keys of an array
 * @link https://php.net/manual/en/function.array-keys.php
 * @param array $array <p>
 * An array containing keys to return.
 * </p>
 * @param mixed $filter_value [optional] <p>
 * If specified, then only keys containing these values are returned.
 * </p>
 * @param bool $strict [optional] <p>
 * Determines if strict comparison (===) should be used during the search.
 * </p>
 * @return int[]|string[] an array of all the keys in input.
 */
#[Pure]
function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array {}

/**
 * Applies the callback to the elements of the given arrays
 * @link https://php.net/manual/en/function.array-map.php
 * @param callable|null $callback <p>
 * Callback function to run for each element in each array.
 * </p>
 * @param array $array <p>
 * An array to run through the callback function.
 * </p>
 * @param array ...$arrays
 * @return array an array containing all the elements of arr1
 * after applying the callback function to each one.
 * @meta
 */
function array_map(?callable $callback, array $array, array ...$arrays): array {}
