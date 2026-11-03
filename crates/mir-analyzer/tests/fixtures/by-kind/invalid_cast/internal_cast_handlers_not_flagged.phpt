===description===
GMP converts to string, int and float and CurlHandle and CurlMultiHandle cast to an int id, through internal cast handlers; CurlShareHandle has none.
===file===
<?php
function gmpToString(\GMP $n): string {
    return (string) $n;
}
function gmpToInt(\GMP $n): int {
    return (int) $n;
}
function gmpToFloat(\GMP $n): float {
    return (float) $n;
}
function curlId(\CurlHandle $h): int {
    return (int) $h;
}
function multiId(\CurlMultiHandle $h): int {
    return (int) $h;
}
function shareId(\CurlShareHandle $h): int {
    return (int) $h;
//               ^^ InvalidCast: Cannot cast 'CurlShareHandle' to 'int'
}
