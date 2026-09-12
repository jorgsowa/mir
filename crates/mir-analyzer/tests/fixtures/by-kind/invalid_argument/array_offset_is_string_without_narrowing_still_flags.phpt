===description===
A string-only call on an unguarded string-or-int array offset is invalid.
===file===
<?php
/** @param array{v: string|int} $arr */
function test(array $arr): string {
    return strtoupper($arr['v']);
}
===expect===
PossiblyInvalidArgument@4:22-4:31: Argument $string of strtoupper() expects 'string', possibly different type 'string|int' provided
