<?php

// Start of bz2 v.1.0

/**
 * Compress a string into bzip2 encoded data
 * @link https://php.net/manual/en/function.bzcompress.php
 * @param string $data <p>The string to compress.</p>
 * @param int $block_size [optional] <p>Specifies the blocksize used during compression. Defaults to 4.</p>
 * @param int $work_factor [optional] <p>Controls how the compression phase behaves when presented with worst case, highly repetitive, input data. Defaults to 0.</p>
 * @return string|int The compressed string, or an integer error code on failure.
 */
function bzcompress(string $data, int $block_size = 4, int $work_factor = 0): string|int {}

/**
 * Decompresses bzip2 encoded data
 * @link https://php.net/manual/en/function.bzdecompress.php
 * @param string $data <p>The string to decompress.</p>
 * @param bool $use_less_memory [optional] <p>Use a slower, but less memory intensive algorithm.</p>
 * @return string|int|false The decompressed string, an integer error code on failure, or false on error.
 */
function bzdecompress(string $data, bool $use_less_memory = false): string|int|false {}

/**
 * Opens a bzip2 compressed file
 * @link https://php.net/manual/en/function.bzopen.php
 * @param string|resource $file <p>Either the name of the file, or an existing fopen() resource.</p>
 * @param string $mode <p>The modes 'r' (read), and 'w' (write) are supported.</p>
 * @return resource|false The file pointer resource, or false on error.
 */
function bzopen($file, string $mode) {}

/**
 * Binary safe bzip2 file read
 * @link https://php.net/manual/en/function.bzread.php
 * @param resource $bz <p>The file pointer.</p>
 * @param int $length [optional] <p>The number of bytes to read.</p>
 * @return string|false The uncompressed data, or false on error.
 */
function bzread($bz, int $length = 1024): string|false {}

/**
 * Binary safe bzip2 file write
 * @link https://php.net/manual/en/function.bzwrite.php
 * @param resource $bz <p>The file pointer.</p>
 * @param string $data <p>The written data.</p>
 * @param int|null $length [optional] <p>If supplied, writing will stop after length bytes have been written.</p>
 * @return int|false The number of bytes written, or false on error.
 */
function bzwrite($bz, string $data, ?int $length = null): int|false {}

/**
 * Close a bzip2 file
 * @link https://php.net/manual/en/function.bzclose.php
 * @param resource $bz <p>The file pointer.</p>
 * @return bool true on success or false on failure.
 */
function bzclose($bz): bool {}

/**
 * Force a write of all buffered data
 * @link https://php.net/manual/en/function.bzflush.php
 * @param resource $bz <p>The file pointer.</p>
 * @return bool true on success or false on failure.
 */
function bzflush($bz): bool {}

/**
 * Returns a bzip2 error string
 * @link https://php.net/manual/en/function.bzerrstr.php
 * @param resource $bz <p>The file pointer.</p>
 * @return string A human readable error message.
 */
function bzerrstr($bz): string {}

/**
 * Returns the bzip2 error number
 * @link https://php.net/manual/en/function.bzerrno.php
 * @param resource $bz <p>The file pointer.</p>
 * @return int The error number as an integer.
 */
function bzerrno($bz): int {}

/**
 * Returns the bzip2 error number and error string in an array
 * @link https://php.net/manual/en/function.bzerror.php
 * @param resource $bz <p>The file pointer.</p>
 * @return array An associative array, with the error code in the "errno" key, and the error message in the "errstr" key.
 */
function bzerror($bz): array {}

// End of bz2 v.1.0
