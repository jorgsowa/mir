===description===
Closure::bind preserves return type from original closure
===file===
<?php
class A {}
class B {}

$closure = function(): string { return "hello"; };
/** @mir-check $closure is Closure(): string */

$bound = Closure::bind($closure, new B());
// bound() should return string (or null if bind failed), not mixed
/** @mir-check $bound is Closure(): string|null */

if ($bound) {
    $result = $bound();
    /** @mir-check $result is string */
}
===expect===
