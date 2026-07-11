===description===
Regression guard for the `$b?->value !== null` narrowing fix: the branch
where `$b?->value === null` is *true* must NOT narrow the receiver `$b` to
non-null — a null receiver is one of the two ways this can be true (the
other being a non-null receiver with a genuinely null `value`), so `$b`
is still possibly null inside this branch.
===config===
suppress=UnusedVariable
===file===
<?php
class Box {
    public ?string $value = null;
    public string $other = "x";
}

function test(?Box $b): void {
    if ($b?->value === null) {
        echo $b->other;
    }
}
===expect===
PossiblyNullPropertyFetch@9:13-9:22: Cannot access property $other on possibly null value
