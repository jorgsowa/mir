===file===
<?php
function typeFunction(object|string $x): object {
    $is = is_object($x);
    return $is ? $x : new stdClass();
}

function negatedTypeFunction(int|string $x): void {
    $is = \is_string($x);
    if (!$is) {
        /** @mir-check $x is int */
        echo 1;
    }
}

function nullComparison(?int $x, ?int $y): void {
    $notNull = null !== $x;
    if ($notNull) {
        /** @mir-check $x is int */
        echo 1;
    }
    $isNull = $y === null;
    if (!$isNull) {
        /** @mir-check $y is int */
        echo 1;
    }
}

function staleAfterWrite(int|string $x): void {
    $is = is_string($x);
    $x = 1;
    if ($is) {
        /** @mir-check $x is 1 */
        echo 1;
    }
}
