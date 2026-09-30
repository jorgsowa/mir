===description===
Int-backed enum ::from() rejects string argument
===config===
php_version=8.1
===file===
<?php
enum Priority: int {
    case Low = 1;
    case High = 2;
}

Priority::from('high');
//             ^^^^^^ InvalidArgument: Argument $value of from() expects 'int', got '"high"'
===expect===
