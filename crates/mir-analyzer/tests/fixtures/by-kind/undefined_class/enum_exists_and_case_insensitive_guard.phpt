===description===
enum_exists() guard suppresses UndefinedClass like class_exists(); the guard
match is case-insensitive.
===file===
<?php
function test_enum_exists_guard(): void {
    if (enum_exists(\App\Suit::class)) {
        new \App\Suit();
    }
}

function test_mixed_case_class_exists_guard(): void {
    if (Class_Exists(\App\Widget::class)) {
        new \App\Widget();
    }
}

function test_no_guard_still_flags(): void {
    new \App\NoGuard();
}
===expect===
WrongCaseFunction@9:8-9:20: Function name 'Class_Exists' has incorrect casing; use 'class_exists'
UndefinedClass@15:8-15:20: Class App\NoGuard does not exist
