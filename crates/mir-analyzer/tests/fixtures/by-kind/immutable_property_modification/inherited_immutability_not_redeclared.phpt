===description===
`@psalm-immutable` was read straight off `ClassDef::is_immutable` (own
declaration only, no ancestor walk) at both consumer sites — a subclass
that doesn't redeclare the tag allowed a `$this`-write from one of its own
non-constructor methods, and an external write through the subclass-typed
receiver, that the ancestor-typed receiver already correctly rejected.
===file===
<?php
/** @psalm-immutable */
class Base {
    public function __construct(
        public float $x,
    ) {}
}

class Sub extends Base {
    public function mutate(): void {
        $this->x = 0.0;
    }
}

function mutateExternally(Sub $s): void {
    $s->x = 1.0;
}
===expect===
ImmutablePropertyModification@11:8-11:22: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@16:4-16:15: Assigning to property x of $s in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
