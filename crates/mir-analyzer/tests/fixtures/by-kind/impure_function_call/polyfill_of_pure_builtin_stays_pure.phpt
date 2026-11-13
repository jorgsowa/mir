===description===
A polyfill that redeclares a pure built-in (`str_contains`, `mb_strlen`) behind a
`function_exists()` check is as pure as the built-in, also from another file. A
polyfill of an impure built-in and a plain user function are still impure.
===file:polyfills.php===
<?php
if (!function_exists('str_contains')) {
    function str_contains(?string $haystack, ?string $needle): bool { return true; }
}
if (!function_exists('mb_strlen')) {
    function mb_strlen(?string $string, ?string $encoding = null): int { return strlen((string) $string); }
}
if (!function_exists('error_log')) {
    function error_log(string $message): bool { return true; }
}
function my_len(string $s): int { return strlen($s); }
===file:use.php===
<?php
/** @psalm-pure */
function has_x(string $s): bool { return str_contains($s, 'x'); }

/** @psalm-pure */
function len(string $s): int { return \mb_strlen($s, 'UTF-8'); }

/** @psalm-pure */
function log_it(string $s): bool { return error_log($s); }
//                                        ^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function error_log() in a @pure function

/** @psalm-pure */
function mine(string $s): int { return my_len($s); }
//                                     ^^^^^^^^^^ ImpureFunctionCall: Calling impure function my_len() in a @pure function
