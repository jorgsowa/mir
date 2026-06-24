===description===
String-backed enum ::from() rejects int argument
===config===
php_version=8.1
===file===
<?php
enum Color: string {
    case Red = 'r';
    case Green = 'g';
}

Color::from(42);
===expect===
ArgumentTypeCoercion@7:12-7:14: Argument $value of from() expects 'string', got '42' — coercion may fail at runtime
