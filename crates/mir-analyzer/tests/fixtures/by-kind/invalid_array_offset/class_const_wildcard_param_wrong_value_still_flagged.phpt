===description===
Resolving `Other::*` keeps argument checks strict: values outside the constants are rejected.
===file===
<?php
final class Kind {
    public const string A = 'a';
    public const string B = 'b';
}
/** @param Kind::* $k */
function pick(string $k): string { return $k; }
$ok = pick(Kind::A);
/** @mir-check $ok is string */
echo $ok;
pick('b');
pick('zzz');
//   ^^^^^ InvalidArgument: Argument $k of pick() expects '"a"|"b"', got '"zzz"'
