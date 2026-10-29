===description===
`is_array()` on a `callable` keeps its `[class, method]` form, which `!is_callable()`
then rules out; a plain array stays, as it may not be callable.
===file===
<?php
/**
 * @param list<string>|callable $by
 */
function sort_by($by): void {
    if (is_array($by) && !is_callable($by)) {
        /** @trace $by */
        echo 1;
//      ^^^^^^^ Trace: Type of $by is list<string>
    }
    if (is_array($by)) {
        /** @trace $by */
        echo 1;
//      ^^^^^^^ Trace: Type of $by is list<string>|array{0: class-string|object, 1: non-empty-string}
        if (!is_callable($by)) {
            /** @trace $by */
            echo 1;
//          ^^^^^^^ Trace: Type of $by is list<string>
        } else {
            /** @trace $by */
            echo 1;
//          ^^^^^^^ Trace: Type of $by is list<string>|array{0: class-string|object, 1: non-empty-string}
        }
    }
}

/**
 * @param array<string, int>|callable $c
 */
function not_callable($c): void {
    if (!is_callable($c)) {
        /** @trace $c */
        echo 1;
//      ^^^^^^^ Trace: Type of $c is array<string, int>
    }
}
