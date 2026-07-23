===description===
`unset($this->prop)` mutates the property just as much as a plain
`$this->prop = x` assignment, but analyze_unset_stmt only ever did a
read-oriented existence check, never running the immutable check.
===file===
<?php

/** @psalm-immutable */
class Bag {
    public ?int $x = 1;

    public function clear(): void {
        unset($this->x);
    }
}
===expect===
ImmutablePropertyModification@8:14-8:22: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
