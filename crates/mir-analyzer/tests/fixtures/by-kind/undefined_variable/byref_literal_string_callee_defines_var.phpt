===description===
Variable-held literal function name premarks by-ref out-param
===file===
<?php
function a(string $re, string $s): int {
    $fn = 'preg_match';
    $fn($re, $s, $m);
    /** @mir-check $m is list<string> */
    return count($m);
}
