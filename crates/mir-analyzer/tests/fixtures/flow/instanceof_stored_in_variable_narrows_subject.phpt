===file===
<?php
class A {}
class B extends A { public function b(): void {} }

function direct(A $x): ?B {
    $is = $x instanceof B;
    return $is ? $x : null;
}

function negated(A|string $x): void {
    $is = $x instanceof A;
    if (!$is) {
        /** @mir-check $x is string */
        echo 1;
    }
}
