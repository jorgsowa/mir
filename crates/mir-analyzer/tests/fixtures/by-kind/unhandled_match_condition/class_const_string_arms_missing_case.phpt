===description===
UnhandledMatchCondition still fires when a class-constant arm leaves a literal uncovered.
===file===
<?php
class C {
    const A = 'a';
    const B = 'b';
}
/** @param 'a'|'b' $x */
function f(string $x): string {
    return match ($x) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: "b"
        C::A => 'x',
    };
}
===expect===
