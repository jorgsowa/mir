===description===
String-backed enum ::tryFrom() returns enum|null, not mixed.
Expected: no issue.
===config===
php_version=8.1
suppress=UnusedVariable
===file===
<?php
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}

$s = Status::tryFrom('active');
/** @mir-check $s is Status|null */
if ($s !== null) {
    echo $s->value;
}
===expect===
