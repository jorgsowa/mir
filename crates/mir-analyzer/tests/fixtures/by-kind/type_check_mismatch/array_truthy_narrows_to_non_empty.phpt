===description===
Truthy check on array<K,V> narrows to non-empty-array<K,V>; likewise for list.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string> $items */
function test_array(array $items): void {
    if ($items) {
        /** @mir-check $items is non-empty-array<int|string, string> */
        $_ = $items;
    } else {
        /** @mir-check $items is array<int|string, string> */
        $_ = $items;
    }
}

/** @param list<int> $nums */
function test_list(array $nums): void {
    if ($nums) {
        /** @mir-check $nums is non-empty-list<int> */
        $_ = $nums;
    } else {
        /** @mir-check $nums is list<int> */
        $_ = $nums;
    }
}
===expect===
