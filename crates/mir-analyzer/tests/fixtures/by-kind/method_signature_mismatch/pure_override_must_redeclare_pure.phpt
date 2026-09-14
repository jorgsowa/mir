===description===
An inherited @pure contract applies to an override without requiring a
redundant child annotation. The child body must still be checked as pure,
which prevents this from regressing into an unchecked implementation.
===file===
<?php
interface Calculator {
    /** @pure */
    public function add(int $a, int $b): int;
}
class Impure implements Calculator {
    public int $calls = 0;
    public function add(int $a, int $b): int {
        $this->calls++;
        return $a + $b;
    }
}
===expect===
ImmutablePropertyModification@9:9-9:23: Assigning to property calls of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
