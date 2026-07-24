===description===
Same cross-class immutable-write gap as the array-index-write sibling,
but for a by-ref call argument (`sort($b->items)`) through a non-`$this`
receiver.
===config===
suppress=MixedArgument
===file===
<?php
/** @psalm-immutable */
class Box {
    public function __construct(
        public array $items,
    ) {}
}

function mutateImmutable(Box $b): void {
    sort($b->items);
}
===expect===
ImmutablePropertyModification@10:9-10:18: Assigning to property items of $b in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
