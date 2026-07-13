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
        C::A => 'x',
    };
}
===expect===
UnhandledMatchCondition@8:11-10:5: Unhandled match condition: "b"
