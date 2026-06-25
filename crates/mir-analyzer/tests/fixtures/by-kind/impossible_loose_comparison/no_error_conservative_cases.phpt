===description===
Conservative: no warning for open types (mixed), object vs true,
general arrays vs false/true, and scalar vs scalar coercions.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj, array $arr, mixed $m, bool $b): void {
    // object == true: always true, but we conservatively skip "always true" detection
    if ($obj == true) {}
    // mixed: open type — conservative, never flag
    if ($m == null) {}
    if ($m == $obj) {}
    // general array vs false: possible (empty array == false is true)
    if ($arr == false) {}
    // general array vs bool: possible
    if ($arr == $b) {}
    // scalar vs scalar: complex coercion, conservative
    if ($m == 0) {}
}
===expect===
