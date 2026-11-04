===description===
`($x ?? false) !== false` proves `$x` is neither null nor false
===file===
<?php
function f(int|false|null $length): int {
    if (false === ($length ?? false)) {
        return 0;
    }
    /** @mir-check $length is int */
    return $length;
}
