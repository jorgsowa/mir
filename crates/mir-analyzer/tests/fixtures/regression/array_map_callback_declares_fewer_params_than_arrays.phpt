===description===
PHP callbacks may always declare fewer params than the caller passes — extra
invocation arguments are simply dropped, not an error. array_map(callback, array1,
array2, ...) invokes callback with one element per array; mir previously required
the callback to declare AT LEAST as many params as arrays passed, which is backwards.
===config===
suppress=UnusedParam
===file===
<?php
function acceptsNone(): bool {
    return true;
}
function acceptsOne(int $a): void {}

array_map("acceptsNone", [1, 2, 3]); // 1 array, 0-param callback
array_map("acceptsOne", [1, 2, 3], [4, 5, 6]); // 2 arrays, 1-param callback

===expect===
