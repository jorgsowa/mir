===description===
Only a write from INSIDE an immutable class's own methods was checked
(ctx.is_in_immutable_method, gated on a literal $this receiver) — an
outside `$b->x = 1;` on an @psalm-immutable-tagged object was a silent
no-op. A plain (non-immutable) receiver's own property write must stay
unflagged, so it's included as a same-shape contrast case.
===file===
<?php
/** @psalm-immutable */
class Box {
    public function __construct(
        public int $x,
    ) {}
}

class MutableBox {
    public function __construct(
        public int $x,
    ) {}
}

function mutateImmutable(Box $b): void {
    $b->x = 1;
}

function mutateMutable(MutableBox $b): void {
    $b->x = 1;
}
===expect===
ImmutablePropertyModification@16:4-16:13: Assigning to property x of $b in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
