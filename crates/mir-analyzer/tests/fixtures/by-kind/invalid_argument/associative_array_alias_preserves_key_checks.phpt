===description===
`associative-array` is only an alias of `array`, not an escape hatch from
ordinary element-type checking: a list with string values still does not
satisfy `associative-array<array-key, int>`.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param associative-array<array-key, int> $items
 */
function takesAssociativeArrayOfInt(array $items): void {}

takesAssociativeArrayOfInt(['x']);
===expect===
InvalidArgument@7:27-7:32: Argument $items of takesAssociativeArrayOfInt() expects 'array<int|string, int>', got 'array{0: "x"}'
