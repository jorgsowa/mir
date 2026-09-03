===description===
FP-P13: count_chars($s, $mode)'s stub declares a flat `array|string` union
regardless of $mode. Per the manual, mode 3 or 4 always returns a string and
mode 0/1/2 always returns an array — narrow to the precise type when $mode
is a literal int, mirroring the preg_split/filter_var literal-argument
narrowing pattern.
===config===
suppress=UnusedParam
===file===
<?php
function needs_string(string $s): void {}

// Real-world repro: mode 3 fed straight into a string-only parameter used
// to raise a PossiblyInvalidArgument false positive.
function unique_chars(string $s): string {
    $unique = count_chars($s, 3);
    needs_string($unique);
    return $unique;
}

function test_mode3_is_string(string $s): void {
    $v = count_chars($s, 3);
    /** @mir-check $v is string */
    $_ = $v;
}

function test_mode4_is_string(string $s): void {
    $v = count_chars($s, 4);
    /** @mir-check $v is string */
    $_ = $v;
}

function test_mode0_is_array(string $s): void {
    $v = count_chars($s, 0);
    /** @mir-check $v is array<int|string, int> */
    $_ = $v;
}

function test_default_mode_is_array(string $s): void {
    $v = count_chars($s);
    /** @mir-check $v is array<int|string, int> */
    $_ = $v;
}

function test_dynamic_mode_falls_back_to_stub(string $s, int $mode): void {
    $v = count_chars($s, $mode);
    /** @mir-check $v is array<int|string, int>|string */
    $_ = $v;
}
===expect===
