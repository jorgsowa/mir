===description===
Negative control: a genuinely unmapped Psalm-only kind name (no entry in the alias
table) must still be flagged `UnusedSuppress` when it matches nothing — proves the
alias table only widens matching for the specific names it lists, not third-party
identifiers in general.
===file===
<?php
class Foo {}
/**
 * @psalm-suppress UndefinedClass, PossiblyInvalidCast
 */
function test(Foo $f): void {
    echo get_class($f);
    new NoSuchClass();
}
===expect===
UnusedSuppress@6:0-6:0: Suppress annotation for 'PossiblyInvalidCast' is never used
