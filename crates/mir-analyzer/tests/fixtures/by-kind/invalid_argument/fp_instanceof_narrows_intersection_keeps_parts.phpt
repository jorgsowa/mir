===description===
FP: instanceof-narrowing an intersection type onto an extra interface must not drop its other parts
===file===
<?php
interface Countable2 {
    public function count2(): int;
}
interface Iterator2 {
    public function next2(): void;
}
interface ArrayAccess2 {
    public function has2(): bool;
}

function needsBoth(Countable2&Iterator2 $y): void {}

function f(Countable2&Iterator2 $x): void {
    if ($x instanceof ArrayAccess2) {
        needsBoth($x);
    }
}
===expect===
UnusedParam@12:19-12:42: Parameter $y is never used
