===description===
++/-- on a property is a write, same as +=, but bypassed purity and
immutability enforcement entirely -- neither analyze_unary_prefix nor
analyze_unary_postfix ever routed a PropertyAccess operand through
check_property_write_purity, unlike the compound-assignment form.
===file===
<?php
class Counter {
    public int $n = 0;

    /** @pure */
    public function bumpParam(Counter $c): void {
        $c->n++;
//      ^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
    }
}

/** @psalm-immutable */
class Frozen {
    public int $n = 0;

    public function bump(): void {
        $this->n++;
//      ^^^^^^^^ ImmutablePropertyModification: Assigning to property n of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
        --$this->n;
//        ^^^^^^^^ ImmutablePropertyModification: Assigning to property n of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
===expect===
