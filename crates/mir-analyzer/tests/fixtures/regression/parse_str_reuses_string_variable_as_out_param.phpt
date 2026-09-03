===description===
FP-L32: `parse_str()`'s second parameter is a pure out-param — PHP's real
signature places no type constraint on the incoming value at all, so
reusing the same variable (`parse_str($s, $s)`) is legal and idiomatic
(verified live). The stub's `@param array &$result` was also used as the
INCOMING type check, flagging InvalidArgument for the reused string. Fixed
via `@param-out array $result`: the in-type is `mixed` (never checked
meaningfully), the out-type (used for writeback) stays `array`.
===file===
<?php
function parseQuery(string $s): array {
    parse_str($s, $s);
    /** @mir-check $s is array */
    return $s;
}
===expect===
