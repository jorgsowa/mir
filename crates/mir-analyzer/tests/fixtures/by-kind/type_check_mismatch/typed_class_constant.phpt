===description===
N3: PHP 8.3 typed class constants — the declared type hint must be used when
resolving Foo::CONST accesses, not mixed. This enables proper type-checking of
constants and allows downstream InvalidArgument to fire when the wrong type is used.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.3
===file===
<?php

class Config {
    const int MAX_RETRIES = 3;
    const string PREFIX = 'app_';
    const float PI = 3.14159;
    const bool ENABLED = true;
}

function test_int_constant(): void {
    $v = Config::MAX_RETRIES;
    /** @mir-check $v is int */
    $_ = $v;
}

function test_string_constant(): void {
    $v = Config::PREFIX;
    /** @mir-check $v is string */
    $_ = $v;
}

function test_float_constant(): void {
    $v = Config::PI;
    /** @mir-check $v is float */
    $_ = $v;
}

function test_bool_constant(): void {
    $v = Config::ENABLED;
    /** @mir-check $v is bool */
    $_ = $v;
}

function needs_string(string $s): void {}

function test_typed_const_triggers_invalid_argument(): void {
    needs_string(Config::MAX_RETRIES);
}
===expect===
ArgumentTypeCoercion@37:17-37:36: Argument $s of needs_string() expects 'string', got 'int' — coercion may fail at runtime
