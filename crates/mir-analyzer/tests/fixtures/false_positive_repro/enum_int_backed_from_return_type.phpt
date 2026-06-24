===description===
Int-backed enum ::from() returns the enum type, not mixed.
Expected: no issue.
===config===
php_version=8.1
suppress=UnusedVariable
===file===
<?php
enum Priority: int {
    case Low = 1;
    case High = 2;
}

$p = Priority::from(1);
/** @mir-check $p is Priority */
echo $p->value;
===expect===
