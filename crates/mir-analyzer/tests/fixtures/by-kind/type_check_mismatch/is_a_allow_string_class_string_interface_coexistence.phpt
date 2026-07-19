===description===
is_a($x, $class, true) true branch no longer drops a class-string<C> atom
just because C isn't provably related to $class — when either C or $class
is an interface, a single class could still implement both, mirroring the
object-atom coexistence check just above this one in the same file.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Cacheable {}
class Foo {}
class Bar {}
class Widget {}

// A real (non-string) object atom is mixed into the union so the
// partition's string_part is combined with narrow_instanceof_preserving_subtypes's
// result rather than short-circuiting on an unchanged/empty type — that
// would otherwise make a wrongly-emptied string_part indistinguishable
// from a correctly-preserved one (both look like "nothing changed").
/** @param class-string<Foo>|class-string<Bar>|Widget $cls */
function test_keeps_unrelated_class_strings_when_checked_interface_could_coexist($cls): void {
    if (is_a($cls, 'Cacheable', true)) {
        // Cacheable is an interface: a subclass of Foo or Bar could still
        // implement it, so neither class-string is provably excluded.
        /** @mir-check $cls is class-string<Foo>|class-string<Bar>|(Widget&Cacheable) */
        $_ = $cls;
    }
}

/** @param class-string<Cacheable>|class-string<Bar> $cls */
function test_keeps_interface_class_string_against_concrete_check(string $cls): void {
    if (is_a($cls, 'Bar', true)) {
        // Cacheable is an interface: some class implementing it could also
        // extend Bar, so class-string<Cacheable> isn't provably excluded.
        /** @mir-check $cls is class-string<Cacheable>|class-string<Bar> */
        $_ = $cls;
    }
}

/** @param class-string<Foo>|class-string<Bar> $cls */
function test_still_drops_unrelated_class_string_when_both_concrete(string $cls): void {
    if (is_a($cls, 'Bar', true)) {
        // Neither Foo nor Bar is an interface and Foo doesn't extend Bar —
        // Foo can never coexist with Bar, so it's still dropped.
        /** @mir-check $cls is class-string<Bar> */
        $_ = $cls;
    }
}
===expect===
