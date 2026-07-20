===description===
str_contains($haystack, Holder::$needle) narrows the haystack to
non-empty-string when the needle is a static property already narrowed to
a single non-empty string literal — expr_is_nonempty_string_literal only
had var/instance-prop arms via ScalarArgTarget, no static-prop arm.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingThrowsDocblock
===file===
<?php
class Holder {
    /** @var 'x' */
    public static string $needle = 'x';

    /** @var '' */
    public static string $emptyNeedle = '';
}

function test_str_contains_static_prop_needle(string $haystack): void {
    if (str_contains($haystack, Holder::$needle)) {
        /** @mir-check $haystack is non-empty-string */
        $_ = $haystack;
    }
}

function test_strpos_static_prop_needle(string $haystack): void {
    if (strpos($haystack, Holder::$needle) !== false) {
        /** @mir-check $haystack is non-empty-string */
        $_ = $haystack;
    }
}

function test_empty_static_prop_needle_not_narrowed(string $haystack): void {
    if (str_contains($haystack, Holder::$emptyNeedle)) {
        /** @mir-check $haystack is string */
        $_ = $haystack;
    }
}
===expect===
