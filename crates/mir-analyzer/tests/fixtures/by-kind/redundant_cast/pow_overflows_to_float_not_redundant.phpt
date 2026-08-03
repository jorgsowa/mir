===description===
M11: `**` (Pow) can overflow int to float at runtime exactly like `/`
does (`2 ** 63` is `float`, not `int`) — it must type as `int|float` for
two int-like operands, not pure `int`, so an overflow-guard `(int)` cast
isn't flagged redundant.
===config===
suppress=UnusedParam
===file===
<?php
function backoff(int $retries): int {
    return (int) (2 ** ($retries - 1));
}
===expect===
