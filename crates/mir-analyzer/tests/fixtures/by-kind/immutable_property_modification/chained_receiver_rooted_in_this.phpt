===description===
$this->cache->v = 5 (a chained, non-$this-literal receiver) escaped
@psalm-immutable checks entirely -- check_property_write_purity only
matched when pa.object was LITERALLY `$this`, never when `$this` was
reached through an intermediate property in the chain.
===config===
suppress=MissingConstructor
===file===
<?php
class Cache {
    public int $v = 0;
}

/** @psalm-immutable */
class Wrapper {
    public Cache $cache;

    public function mutate(): void {
        $this->cache->v = 5;
    }
}
===expect===
ImmutablePropertyModification@11:8-11:27: Assigning to property v of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
