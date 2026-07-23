===description===
$this->caches[0]->v = 5 (an array-index hop in the middle of the chain)
still escaped @psalm-immutable checks -- root_receiver_var's chain-walk
only recursed through PropertyAccess/NullsafePropertyAccess, so an
ArrayAccess node in the middle made it bail out to None.
===config===
suppress=MissingConstructor
===file===
<?php
class Cache {
    public int $v = 0;
}

/** @psalm-immutable */
class Wrapper {
    /** @var Cache[] */
    public array $caches;

    public function mutate(): void {
        $this->caches[0]->v = 5;
    }
}
===expect===
ImmutablePropertyModification@12:8-12:31: Assigning to property v of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
