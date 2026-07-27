===description===
Same named-argument-reordering bug as the free-function case, for an
instance method call's by-ref write-back loop (`call/method.rs`) — a
by-ref target passed via a named argument out of declared order was
checked against the wrong argument instead of the real by-ref target.
===config===
suppress=MissingConstructor,MixedArgument,MixedArrayAssignment,UnusedParam
===file===
<?php
class Bag {
    public array $items = [];
}

class Filler {
    public function fill(int $skip, array &$out): void {
        $out[] = 1;
    }
}

/** @psalm-immutable */
class Holder {
    public Bag $bag;

    public function corrupt(Filler $filler): void {
        $filler->fill(out: $this->bag->items, skip: 1);
    }
}
===expect===
ImmutablePropertyModification@17:27-17:44: Assigning to property items of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
