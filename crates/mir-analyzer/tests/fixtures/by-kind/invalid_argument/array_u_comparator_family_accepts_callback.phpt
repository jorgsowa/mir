===description===
The `array_u*`/`*_ukey`/`*_uassoc` family's trailing comparator-callback
argument(s) must not be rejected as `array` — phpstorm-stubs' `@param array
...$rest` docblock mistypes the actual runtime slot, which is always a
callback.
===config===
suppress=MissingClosureReturnType
===file===
<?php
$a = ['a' => 1];
$b = ['b' => 2];
$cmp = function ($x, $y) { return $x <=> $y; };

array_udiff($a, $b, $cmp);
array_uintersect($a, $b, $cmp);
array_uintersect_assoc($a, $b, $cmp);
array_intersect_uassoc($a, $b, $cmp);
array_uintersect_uassoc($a, $b, $cmp, $cmp);
array_diff_ukey($a, $b, $cmp);
array_intersect_ukey($a, $b, $cmp);
array_udiff_assoc($a, $b, $cmp);
array_diff_uassoc($a, $b, $cmp);
array_udiff_uassoc($a, $b, $cmp, $cmp);
array_udiff($a, $b, 'strcmp');
===expect===
