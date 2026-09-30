===description===
Int-backed enum ::tryFrom() rejects string argument
===config===
php_version=8.1
===file===
<?php
enum Priority: int {
    case Low = 1;
}

Priority::tryFrom('low');
//                ^^^^^ InvalidArgument: Argument $value of tryFrom() expects 'int', got '"low"'
===expect===
