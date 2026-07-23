===description===
An array-index write through $this->prop (`$this->items[] = x`,
`$this->items['k'] = x`) mutates the property in place just as much as a
plain `$this->prop = x` assignment, but the array-index write path resolved
a non-variable base as a read-only reference and never ran the immutable
check.
===file===
<?php

/** @psalm-immutable */
class Bag {
    /** @var array<int> */
    public array $items = [];

    public function push(int $n): void {
        $this->items[] = $n;
    }

    public function setKey(int $n): void {
        $this->items['k'] = $n;
    }
}
===expect===
ImmutablePropertyModification@9:8-9:27: Assigning to property items of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@13:8-13:30: Assigning to property items of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
