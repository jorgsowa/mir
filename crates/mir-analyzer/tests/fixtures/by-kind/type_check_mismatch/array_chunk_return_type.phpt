===description===
array_chunk returns list<list<T>> for default preserve_keys=false; outer list is non-empty when source is non-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_chunk_list_default(array $arr): void {
    $chunks = array_chunk($arr, 3);
    /** @mir-check $chunks is list<list<string>> */
    $_ = $chunks;
}

/** @param non-empty-list<int> $arr */
function test_chunk_non_empty_source(array $arr): void {
    $chunks = array_chunk($arr, 2);
    /** @mir-check $chunks is non-empty-list<list<int>> */
    $_ = $chunks;
}
===expect===
