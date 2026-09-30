===description===
String-backed enum ::tryFrom() rejects int argument
===config===
php_version=8.1
===file===
<?php
enum Status: string {
    case Active = 'active';
}

Status::tryFrom(123);
//              ^^^ ArgumentTypeCoercion: Argument $value of tryFrom() expects 'string', got '123' — coercion may fail at runtime
===expect===
