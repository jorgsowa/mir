===description===
A function with @return T (template, effectively mixed) does NOT fire MixedReturnStatement — the declared type T
is itself mixed, so the condition !declared.is_mixed() is false
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @template T
 * @param array<T> $items
 * @return T
 */
function first(array $items) {
    return $items[0];
}
===expect===
