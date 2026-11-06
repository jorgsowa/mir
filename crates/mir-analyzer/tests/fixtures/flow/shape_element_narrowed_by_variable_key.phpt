===description===
A check on `$s[$k]` narrows the element of a closed shape read through that key.
===file===
<?php
/** @param array{a: int|string, b: string|null} $s */
function f(array $s, string $k): void {
    if (is_string($s[$k])) {
        $z = $s[$k];
        /** @mir-check $z is string */
        echo $z;
    }
}
