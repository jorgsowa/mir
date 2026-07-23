===description===
Same chained-receiver gap as the @psalm-immutable case, but for
@psalm-external-mutation-free: calling an impure method on a parameter's
property ($other->cache->bump()) escaped the check because it only ever
matched a literal Variable receiver, never a chained one.
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

class Service {
    /** @psalm-external-mutation-free */
    public function run(Box $other): void {
        $other->cache->bump();
    }
}
===expect===
ImpureMethodCall@14:8-14:29: Calling impure method bump() in a pure or immutable context
