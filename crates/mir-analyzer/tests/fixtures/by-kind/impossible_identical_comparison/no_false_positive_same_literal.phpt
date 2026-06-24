===description===
Identical literals can be ===; no diagnostic should fire.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = 5;
    $b = 5;
    if ($a === $b) {}
    $s1 = "foo";
    $s2 = "foo";
    if ($s1 === $s2) {}
}
===expect===
