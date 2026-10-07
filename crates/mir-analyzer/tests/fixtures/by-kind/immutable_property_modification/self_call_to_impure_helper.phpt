===description===
A `self::` call to a helper of the same @psalm-immutable class is not
flagged: the helper is checked as mutation-free, so its write is reported
where it happens. A `self::` call to a mutable parent's method stays flagged.
===file===
<?php
class MutableBase {
    public int $y = 0;

    public function bump(): void {
        $this->y++;
    }
}

/** @psalm-immutable */
class C extends MutableBase {
    public int $x = 0;

    public function f(): void {
        self::mutateHelper();
        parent::bump();
//      ^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method bump() in a pure or immutable context
    }

    public function mutateHelper(): void {
        $this->x = 1;
//      ^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
