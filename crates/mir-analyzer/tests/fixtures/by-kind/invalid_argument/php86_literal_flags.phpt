===description===
From PHP 8.6 array_filter(), array_change_key_case(), pathinfo() and scandir() throw a ValueError for a flag outside their constants; a value only known as int isn't reported.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function flags(array $a, string $path, int $mode): void {
    array_filter($a, null, 3);
//                         ^ InvalidArgument: Argument $mode of array_filter() expects '0|1|2', got '3'
    array_filter($a, fn($key) => $key !== '', ARRAY_FILTER_USE_KEY);
    array_filter($a, null, $mode);
    array_change_key_case($a, 2);
//                            ^ InvalidArgument: Argument $case of array_change_key_case() expects '0|1', got '2'
    array_change_key_case($a, CASE_UPPER);
    echo pathinfo($path, PATHINFO_DIRNAME | PATHINFO_BASENAME);
//                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $flags of pathinfo() expects '1|2|4|8|15', got '3'
    echo pathinfo($path, PATHINFO_EXTENSION);
    scandir($path, 3);
//                 ^ InvalidArgument: Argument $sorting_order of scandir() expects '0|1|2', got '3'
    scandir($path, SCANDIR_SORT_NONE);
}
