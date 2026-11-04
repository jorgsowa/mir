===description===
sort() turns an array with unknown elements into a list
===file===
<?php
/** @param non-empty-array $values */
function f(array $values, array $any): void {
    sort($values);
    /** @mir-check $values is non-empty-list<mixed> */
    rsort($any);
    /** @mir-check $any is list<mixed> */
    echo 1;
}
