===description===
checkCallableTypeArrayInstanceFirstArg
===file===
<?php
/**
 * @param callable(int,int):int $_p
 */
function f(callable $_p): void {}

class C {
    public static function m(string $a, string $b): int { return $a <=> $b; }
}

f([new C, "m"]);
===expect===
InvalidScalarArgument
===ignore===
TODO
