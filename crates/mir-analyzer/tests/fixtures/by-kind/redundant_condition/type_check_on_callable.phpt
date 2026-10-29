===description===
A `callable` may be a function-name string, a `[class, method]` array or an invokable object, so `is_string()`, `is_array()` and `is_object()` on one can hold; `is_int()` can't.
===file===
<?php
function strings(callable $c): void {
    if (is_string($c)) {
        /** @trace $c */
        echo 1;
//      ^^^^^^^ Trace: Type of $c is callable-string
    }
}

function arrays(?callable $c): void {
    if (is_array($c)) {
        /** @trace $c */
        echo 1;
//      ^^^^^^^ Trace: Type of $c is array{0: class-string|object, 1: non-empty-string}
    }
}

function objects(callable $c): void {
    if (is_object($c)) {
        /** @trace $c */
        echo 1;
//      ^^^^^^^ Trace: Type of $c is object
    }
}

function ints(callable $c): void {
    if (is_int($c)) {
//      ^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo 1;
    }
}

function handler(): mixed {
    $handler = set_error_handler(null);
    return is_array($handler) ? $handler[0] : null;
}
