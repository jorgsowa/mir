===description===
From PHP 8.6 passing an object to array_walk(), array_walk_recursive() or http_build_query() is deprecated; a value that may be an array isn't reported.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function walk(object $o, ArrayObject $ao, array $a, array|object $either): string {
    array_walk($o, fn() => null);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function array_walk: array_walk(): Passing an object for argument #1 $array to array_walk() is deprecated, call get_object_vars() first instead
    array_walk_recursive($ao, fn() => null);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function array_walk_recursive: array_walk_recursive(): Passing an object for argument #1 $array to array_walk_recursive() is deprecated, call get_object_vars() first instead
    array_walk($a, fn() => null);
    array_walk($either, fn() => null);
    return http_build_query($o) . http_build_query($a);
//         ^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function http_build_query: http_build_query(): Passing an object for argument #1 $data to http_build_query() is deprecated, call get_object_vars() first instead
}
