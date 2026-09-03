===description===
Int-backed enum ::tryFrom() returns enum|null, not mixed.
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

$p = Priority::tryFrom(99);
/** @mir-check $p is Priority|null */
if ($p !== null) {
    echo $p->value;
}
===expect===
