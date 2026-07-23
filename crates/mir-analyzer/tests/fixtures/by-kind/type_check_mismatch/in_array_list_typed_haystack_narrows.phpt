===description===
in_array()'s haystack-literal extraction also recognizes a list<T>-typed
haystack (not just a TKeyedArray shape) when its element type is itself a
pure literal union, mirroring array_key_exists()'s TKeyedArray-vs-list
coverage pattern.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<'a'|'b'> $arr */
function test_list_haystack_narrows(string $mode, array $arr): void {
    if (in_array($mode, $arr)) {
        /** @mir-check $mode is "a"|"b" */
        $_ = 1;
    }
}

/** @param non-empty-list<'a'|'b'|'c'> $arr */
function test_non_empty_list_haystack_narrows(string $mode, array $arr): void {
    if (in_array($mode, $arr)) {
        /** @mir-check $mode is "a"|"b"|"c" */
        $_ = 1;
    }
}

/** @param list<string> $arr */
function test_non_literal_list_no_narrow(string $mode, array $arr): void {
    if (in_array($mode, $arr)) {
        /** @mir-check $mode is string */
        $_ = 1;
    }
}
===expect===
