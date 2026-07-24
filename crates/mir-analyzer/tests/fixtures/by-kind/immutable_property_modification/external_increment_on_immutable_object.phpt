===description===
Same cross-class immutable-write gap as the array-index-write sibling,
but for `++` through a non-`$this` receiver (`$b->n++;`) — reaches the
same shared check via `check_byref_arg_purity`'s unary arm, which never
ran the cross-class immutable check either before this fix.
===file===
<?php
/** @psalm-immutable */
class Box {
    public function __construct(
        public int $n,
    ) {}
}

function mutateImmutable(Box $b): void {
    $b->n++;
}
===expect===
ImmutablePropertyModification@10:4-10:9: Assigning to property n of $b in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
