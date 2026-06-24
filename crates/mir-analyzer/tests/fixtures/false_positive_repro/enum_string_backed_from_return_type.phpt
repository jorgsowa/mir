===description===
String-backed enum ::from() returns the enum type, not mixed.
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

$s = Status::from('active');
/** @mir-check $s is Status */
echo $s->value;
===expect===
