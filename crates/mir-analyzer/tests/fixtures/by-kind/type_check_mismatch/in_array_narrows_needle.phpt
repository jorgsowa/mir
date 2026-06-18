===description===
in_array($needle, ['a', 'b', 'c']) true-branch narrows $needle to the literal union.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_string_literal_haystack(string $mode): void {
    if (in_array($mode, ['read', 'write', 'append'])) {
        /** @mir-check $mode is "read"|"write"|"append" */
        $_ = $mode;
    }
}

function test_int_literal_haystack(int $code): void {
    if (in_array($code, [200, 201, 204])) {
        /** @mir-check $code is 200|201|204 */
        $_ = $code;
    }
}
===expect===
