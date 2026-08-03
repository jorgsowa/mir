===description===
An int|float-typed divisor (e.g. from ** — which can overflow int to float)
must not collapse a division's result to bare float — only a value
GUARANTEED float should do that. Otherwise a later strict `!== 0` int
comparison on the (possibly-still-int) result is wrongly flagged always-true.
===config===
suppress=UnusedParam
===file===
<?php
function test(int $maxDigits, int $mul, int $value): void {
    $complement = 10 ** $maxDigits;
    $carry = ($mul - $value) / $complement;
    if ($carry !== 0) {
        echo "nonzero";
    }
}
===expect===
