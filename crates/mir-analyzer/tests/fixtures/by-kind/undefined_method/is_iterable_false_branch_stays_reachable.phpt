===description===
`is_iterable()`'s false branch must not strip every object atom — only a
plain array is guaranteed excluded, so `Box|array` still narrows to `Box` in
the else branch and a real bug there is caught (previously silently skipped
as unreachable).
===config===
suppress=UnusedForeachValue
===file===
<?php
class Box {}

/** @param Box|array<int,int> $x */
function f($x): void {
    if (is_iterable($x)) {
        foreach ($x as $v) {}
    } else {
        $x->method();
    }
}
===expect===
UndefinedMethod@9:8-9:20: Method Box::method() does not exist
