===description===
Closure::bind with null newThis still preserves return type (unbinds closure)
===file===
<?php
$closure = function(): bool { return true; };
/** @mir-check $closure is Closure(): bool */

$unbound = Closure::bind($closure, null);
// Even with null $newThis, the return type should be preserved
/** @mir-check $unbound is Closure(): bool|null */

if ($unbound) {
    $result = $unbound();
    /** @mir-check $result is bool */
}
===expect===
