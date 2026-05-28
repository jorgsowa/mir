===description===
array_keys preserves key type from input array
===file===
<?php
/** @param array<non-empty-string, scalar|null> $conditions */
function check(array $conditions): void {
    foreach (array_keys($conditions) as $key) {
        /** @mir-check $key is non-empty-string */
        trim($key);
    }
}

/** @param array<int, mixed> $list */
function checkInt(array $list): void {
    foreach (array_keys($list) as $k) {
        /** @mir-check $k is int */
        $_ = $k + 1;
    }
}
===expect===
