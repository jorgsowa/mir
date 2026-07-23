===description===
@pure semantically implies no side effects at all, including no `$this`
mutation — but is_in_immutable_method was only ever set for
@mutation-free/@psalm-immutable, not a bare @pure method, so `@pure` alone
let a method mutate `$this` completely unchecked. Sanity check confirms
the identical body under @psalm-mutation-free (already correctly checked)
for comparison.
===file===
<?php
class Counter {
    public int $n = 0;

    /** @pure */
    public function increment(): int {
        $this->n = $this->n + 1;
        return $this->n;
    }

    /** @psalm-mutation-free */
    public function incrementMutationFree(): int {
        $this->n = $this->n + 1;
        return $this->n;
    }
}
===expect===
ImmutablePropertyModification@7:8-7:31: Assigning to property n of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@13:8-13:31: Assigning to property n of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
