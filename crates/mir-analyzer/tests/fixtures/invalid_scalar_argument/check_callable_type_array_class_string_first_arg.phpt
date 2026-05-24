===description===
checkCallableTypeArrayClassStringFirstArg
===file===
<?php
/**
 * @param callable(int,int):int $_p
 */
function f(callable $_p): void {}

class C {
    public static function m(string $a, string $b): int { return $a <=> $b; }
}

f([C::class, "m"]);
===expect===
InvalidScalarArgument
===ignore===
TODO
