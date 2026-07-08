===description===
A match() on a bool-typed subject is exhaustive only when both `true` and
`false` arms are present, since bool has exactly two values. The
match(true)/match(false) chained-condition idiom (subject is a literal
true/false constant, arms are arbitrary boolean expressions) is excluded —
its exhaustiveness depends on the arms' condition coverage, not on literal
true/false arms existing.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
function both_arms(bool $b): string {
    return match ($b) { true => 'yes', false => 'no' };
}

function missing_false(bool $b): string {
    return match ($b) { true => 'yes' };
}

class Foo {}
function match_true_idiom_not_flagged(Foo $foo): int {
    return match (true) {
        $foo instanceof Foo => 1,
    };
}
===expect===
UnhandledMatchCondition@7:11-7:39: Unhandled match condition: false
