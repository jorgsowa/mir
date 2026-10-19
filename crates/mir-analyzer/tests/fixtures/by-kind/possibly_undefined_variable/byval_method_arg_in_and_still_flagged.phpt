===description===
Passing a possibly-undefined variable to a by-value method param doesn't
define it, so it stays possibly undefined in the true branch of `&&`.
===file===
<?php
final class Sink {
    public function take(mixed $v): bool {
        return $v !== null;
    }
}

function f(bool $a, bool $b, Sink $s): mixed {
    if ($a) {
        $v = 1;
    }
    if ($b && $s->take($v)) {
//                     ^^ PossiblyUndefinedVariable: Variable $v might not be defined
        return $v;
//             ^^ PossiblyUndefinedVariable: Variable $v might not be defined
    }
    return null;
}
