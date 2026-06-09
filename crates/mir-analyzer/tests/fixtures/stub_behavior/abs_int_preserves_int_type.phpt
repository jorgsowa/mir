===description===
abs(int) template preserves int type, abs(float) preserves float type

===file===
<?php

function takesInt(int $x): void {}

function test(int $n): void {
    takesInt(abs($n)); // No TypeMismatch expected
}

function testFloat(float $f): void {
    $result = abs($f); // Should be float
    /** @mir-check $result is float */
}

function testInt(int $i): void {
    $result = abs($i); // Should be int
    /** @mir-check $result is int */
}
?>
===expect===
