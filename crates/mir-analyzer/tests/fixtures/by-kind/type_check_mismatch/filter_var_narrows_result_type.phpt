===description===
filter_var($value, FILTER_VALIDATE_*) infers the real result type from a
literal filter constant instead of the stub's blanket `mixed` — int|false,
float|false, bool, string|false depending on the filter. A 3rd (options)
argument bails out to the stub's `mixed`, since FILTER_NULL_ON_FAILURE
(settable there) would add `null` to the failure case.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_int(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_INT);
    /** @mir-check $v is int|false */
    $_ = $v;
}

function test_float(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_FLOAT);
    /** @mir-check $v is float|false */
    $_ = $v;
}

function test_boolean(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_BOOLEAN);
    /** @mir-check $v is bool */
    $_ = $v;
}

function test_email(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_EMAIL);
    /** @mir-check $v is string|false */
    $_ = $v;
}

function test_url(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_URL);
    /** @mir-check $v is string|false */
    $_ = $v;
}

function test_ip(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_IP);
    /** @mir-check $v is string|false */
    $_ = $v;
}

// A 3rd (options) argument may carry FILTER_NULL_ON_FAILURE — bail to mixed.
function test_with_options_bails_to_stub(string $s): void {
    $v = filter_var($s, FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE);
    /** @mir-check $v is mixed */
    $_ = $v;
}
===expect===
MixedAssignment@40:4-40:68: Variable $v is assigned a mixed type
MixedAssignment@42:4-42:11: Variable $_ is assigned a mixed type
