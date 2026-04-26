<?php

const YAML_ANY_ENCODING = 0;
const YAML_UTF8_ENCODING = 1;
const YAML_UTF16LE_ENCODING = 2;
const YAML_UTF16BE_ENCODING = 3;
const YAML_ANY_BREAK = 0;
const YAML_CR_BREAK = 1;
const YAML_LN_BREAK = 2;
const YAML_CRLN_BREAK = 3;

/**
 * Parse a YAML string.
 *
 * @param string   $input     The YAML string to parse.
 * @param int      $pos       Document offset; -1 returns all documents as an array.
 * @param int      $ndocs     Set to the number of documents found in the input.
 * @param array    $callbacks Content handlers indexed by YAML tag.
 * @return mixed Parsed value, or false on error.
 */
function yaml_parse(string $input, int $pos = 0, int &$ndocs = null, array $callbacks = []): mixed {}

/**
 * Parse a YAML file.
 *
 * @param string   $filename  Path to the file to parse.
 * @param int      $pos       Document offset; -1 returns all documents as an array.
 * @param int      $ndocs     Set to the number of documents found in the file.
 * @param array    $callbacks Content handlers indexed by YAML tag.
 * @return mixed Parsed value, or false on error.
 */
function yaml_parse_file(string $filename, int $pos = 0, int &$ndocs = null, array $callbacks = []): mixed {}

/**
 * Parse a YAML URL.
 *
 * @param string   $url       URL to the YAML document.
 * @param int      $pos       Document offset; -1 returns all documents as an array.
 * @param int      $ndocs     Set to the number of documents found.
 * @param array    $callbacks Content handlers indexed by YAML tag.
 * @return mixed Parsed value, or false on error.
 */
function yaml_parse_url(string $url, int $pos = 0, int &$ndocs = null, array $callbacks = []): mixed {}

/**
 * Returns the YAML representation of a value.
 *
 * @param mixed  $data      The data to encode.
 * @param int    $encoding  Output encoding (one of the YAML_*_ENCODING constants).
 * @param int    $linebreak Line-break style (one of the YAML_*_BREAK constants).
 * @param array  $callbacks Content handlers indexed by class name.
 */
function yaml_emit(mixed $data, int $encoding = YAML_ANY_ENCODING, int $linebreak = YAML_ANY_BREAK, array $callbacks = []): string {}

/**
 * Send the YAML representation of a value to a file.
 *
 * @param string $filename  Path of the output file.
 * @param mixed  $data      The data to encode.
 * @param int    $encoding  Output encoding (one of the YAML_*_ENCODING constants).
 * @param int    $linebreak Line-break style (one of the YAML_*_BREAK constants).
 * @param array  $callbacks Content handlers indexed by class name.
 */
function yaml_emit_file(string $filename, mixed $data, int $encoding = YAML_ANY_ENCODING, int $linebreak = YAML_ANY_BREAK, array $callbacks = []): bool {}
