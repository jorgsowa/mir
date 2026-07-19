===description===
`variadic_element_type` unconditionally unwrapped a variadic array param's
value type without checking the key was int-shaped — `@param
array<string, int> ...$maps` (each argument literally IS a string-keyed
array) got misread as "each argument is an int", producing a
false-positive InvalidArgument on a well-typed call and missing the real
mismatch on a badly-typed one.
===config===
suppress=MissingThrowsDocblock,UnusedParam
===file===
<?php
/** @param array<string, int> ...$maps */
function sumMaps(...$maps): int {
    return 0;
}

sumMaps(['a' => 1, 'b' => 2], ['c' => 3]);
sumMaps(5);
===expect===
InvalidArgument@8:8-8:9: Argument $maps of sumMaps() expects 'array<string, int>', got '5'
