===description===
`int<0, max>` and other bounded integer ranges should widen to `float` when passed to
a `float` parameter, just as plain `int` does. Regression: `strlen()` returns
`int<0, max>`, and passing it to `log()` (which takes `float`) was emitting
`InvalidArgument`.
===config===
php_version=8.1
===file===
<?php
declare(strict_types=1);

function alphabet_bits(string $alphabet): int {
    $size = strlen($alphabet);        // int<0, max>
    return (int) ceil(log($size, 2.0)); // int<0, max> -> float param
}

function signed_log(int $n): float {
    return log(abs($n));  // plain int -> float param (already worked)
}

function range_log(string $s): float {
    $len = strlen($s);   // int<0, max>
    return log($len + 1); // int<0, max> + 1 -> float param
}
===expect===
