===description===
Same named-argument-reordering bug as the free-function case, for an
instance method call's by-ref write-back loop (`call/method.rs`) — a
by-ref target passed via a named argument out of declared order was
checked against the wrong argument instead of the real by-ref target.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                         ^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property items of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
===expect===
