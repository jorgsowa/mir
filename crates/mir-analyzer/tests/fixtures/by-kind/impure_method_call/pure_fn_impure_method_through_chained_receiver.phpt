===description===
Same chained-receiver gap for a plain @pure function: calling an impure
method on a parameter's property ($box->cache->bump()) escaped the check
because it only ever matched a literal Variable receiver, never a chained
one.
===config===
suppress=MissingConstructor
===file===
<?php
class Cache {
    public function bump(): void {
    }
}

class Box {
    public Cache $cache;
}

/** @pure */
function run(Box $box): void {
    $box->cache->bump();
}
===expect===
ImpureMethodCall@13:4-13:23: Calling impure method bump() in a pure or immutable context
