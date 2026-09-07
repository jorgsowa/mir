===description===
`is_a($x, ...)`/`is_subclass_of($x, ...)` weren't recognized as a valid
single-receiver leaf by the OR-disjunct union machinery, so a disjunct
containing one bailed out of the whole union: a plain `if` left $x
completely unnarrowed, and switch(true)/match(true) fell back to
narrowing each condition SEQUENTIALLY (AND-composing them instead of
unioning) instead of using the union machinery this fix makes reachable.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
interface Shape {}
final class A implements Shape {}
final class B implements Shape {}
final class C implements Shape {}

// Positive: mixed instanceof + is_a() disjunct on a plain variable.
function mixedInstanceofIsA(Shape $x): void {
    if ($x instanceof A || is_a($x, B::class)) {
        /** @mir-check $x is A|B */
        $_ = $x;
    }
}

// Positive: pure is_a()/is_a() disjunct chain via switch(true) fallthrough —
// previously fell back to narrowing each condition SEQUENTIALLY, silently
// collapsing the result to a single disjunct's narrow instead of the union
// (`A|B`): `set_narrowed` leaves a variable's type unchanged, not emptied,
// when a later narrow in the sequence contradicts the current value, so the
// bug reads as one class going missing, not a crash.
function pureIsASwitchTrue(Shape $x): void {
    switch (true) {
        case is_a($x, A::class):
        case is_a($x, B::class):
            /** @mir-check $x is A|B */
            $_ = $x;
    }
}

// Negative: different variables must not merge.
function differentVars(Shape $x, Shape $y): void {
    if (is_a($x, A::class) || is_a($y, B::class)) {
        /** @mir-check $x is Shape */
        $_ = $x;
    }
}
===expect===
