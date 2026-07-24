===description===
The cross-class immutable-write check (any receiver, not just `$this`)
only ever ran from the plain `$b->x = 1;` assignment arm — an
array-index write through the same non-`$this` receiver
(`$b->items['x'] = 1;`) on a `@psalm-immutable`-tagged object was a
silent no-op.
===file===
<?php
/** @psalm-immutable */
class Box {
    public function __construct(
        public array $items,
    ) {}
}

function mutateImmutable(Box $b): void {
    $b->items['x'] = 1;
}
===expect===
ImmutablePropertyModification@10:4-10:22: Assigning to property items of $b in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
