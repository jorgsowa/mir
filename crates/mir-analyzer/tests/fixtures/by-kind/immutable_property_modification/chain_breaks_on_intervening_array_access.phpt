===description===
$this->caches[0]->v = 5 (an array-index hop in the middle of the chain)
still escaped @psalm-immutable checks -- root_receiver_var's chain-walk
only recursed through PropertyAccess/NullsafePropertyAccess, so an
ArrayAccess node in the middle made it bail out to None.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^^^^^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property v of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
