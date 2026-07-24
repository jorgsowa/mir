===description===
`@return ($this is X ? A : B)`: `$this` was never resolved by the
conditional-return discriminator lookup at all (never a declared
parameter), AND separately a CLASS-typed discriminant subject has no
predicate arm in the purely-structural resolver — both had to be fixed
together for a `$this is ClassName` conditional to ever resolve, instead
of silently widening to the union of both branches.
===config===
suppress=UnusedVariable
===file===
<?php
class Box {
    /** @return ($this is NonEmptyBox ? true : false) */
    public function isNonEmpty(): bool {
        return false;
    }
}
class NonEmptyBox extends Box {}

function checkNonEmpty(NonEmptyBox $b): void {
    $x = $b->isNonEmpty();
    /** @mir-check $x is true */
    $_ = 1;
}

function checkPlain(Box $b): void {
    $y = $b->isNonEmpty();
    /** @mir-check $y is false */
    $_ = 1;
}
===expect===
