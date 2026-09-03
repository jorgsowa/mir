===description===
No UnsupportedReferenceUsage for well-formed lvalue reference assignments.
$x = &$y, $x = &$arr[0], and $x = &$obj->prop are all legal PHP and must not fire.
===config===
suppress=UnusedVariable,MixedAssignment
===file===
<?php
/** Simple variable reference — the canonical FP-L case */
$a = "hello";
$b = &$a;

/** Reference to an array offset */
/** @var array<int, string> */
$arr = [];
$ref = &$arr[0];

/** Reference to an object property */
class Box {
    public string $value = '';
}
$box = new Box();
$ref2 = &$box->value;

/** Reference to a nested array offset */
/** @var array<string, array<int, string>> */
$matrix = [];
$cell = &$matrix['row'][0];

===expect===
