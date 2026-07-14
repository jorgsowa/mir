===description===
array_search($needle, $haystack) !== false narrows $needle like in_array()
does; === false (not found) removes matched literals from a finite union.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_string_literal_haystack(string $mode): void {
    if (array_search($mode, ['read', 'write', 'append']) !== false) {
        /** @mir-check $mode is "read"|"write"|"append" */
        $_ = $mode;
    }
}

function test_int_literal_haystack(int $code): void {
    if (false !== array_search($code, [200, 201, 204])) {
        /** @mir-check $code is 200|201|204 */
        $_ = $code;
    }
}

/** @param "a"|"b"|"c"|"d" $mode */
function test_false_branch_removes_literals(string $mode): void {
    if (array_search($mode, ['a', 'b']) === false) {
        /** @mir-check $mode is "c"|"d" */
        $_ = $mode;
    }
}
===expect===
