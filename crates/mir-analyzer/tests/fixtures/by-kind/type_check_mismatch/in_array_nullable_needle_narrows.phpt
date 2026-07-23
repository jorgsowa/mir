===description===
in_array() without strict comparison can still narrow a nullable string/int
needle — null only loosely matches a falsy haystack literal (0, "", "0"),
so a haystack with none of those proves a match wasn't null. When the
haystack DOES contain a falsy literal, null can't be ruled out and no
narrowing happens.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param "a"|"b"|null $mode */
function test_nullable_narrows_no_falsy_literal(?string $mode): void {
    if (in_array($mode, ['a', 'b'])) {
        /** @mir-check $mode is "a"|"b" */
        $_ = $mode;
    }
}

/** @param "a"|"0"|null $mode */
function test_nullable_no_narrow_with_falsy_literal(?string $mode): void {
    if (in_array($mode, ['a', '0'])) {
        /** @mir-check $mode is "a"|"0"|null */
        $_ = $mode;
    }
}

/** @param 1|2|null $mode */
function test_nullable_int_narrows(?int $mode): void {
    if (in_array($mode, [1, 2])) {
        /** @mir-check $mode is 1|2 */
        $_ = $mode;
    }
}
===expect===
