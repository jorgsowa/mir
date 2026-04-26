<?php

use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

/**
 * Unpack data from binary string
 * @link https://php.net/manual/en/function.unpack.php
 * @param string $format <p>
 * See pack for an explanation of the format codes.
 * </p>
 * @param string $string <p>
 * The packed data.
 * </p>
 * @param int $offset [optional]
 * @return array<int, mixed>|false an associative array containing unpacked elements of binary
 * string or false if the format string contains errors
 */
#[Pure]
function unpack(
    string $format,
    string $string,
    #[PhpStormStubsElementAvailable(from: '7.1')] int $offset = 0
): array|false {}
