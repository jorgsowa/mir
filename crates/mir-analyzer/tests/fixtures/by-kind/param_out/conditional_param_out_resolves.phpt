===description===
A conditional `@param-out` resolves against the argument for functions, methods and static methods
===file===
<?php
/** @param-out ($o is object ? int : string) $o */
function setF(object|array &$o): void { $o = 1; }

final class Acc {
    /** @param-out ($o is object ? int : string) $o */
    public function set(object|array &$o): void { $o = 1; }

    /** @param-out ($o is object ? int : string) $o */
    public static function setS(object|array &$o): void { $o = 1; }
}

function f(Acc $acc, stdClass $a, stdClass $b, stdClass $c): void {
    setF($a);
    /** @mir-check $a is int */
    echo $a;
    $acc->set($b);
    /** @mir-check $b is int */
    echo $b;
    Acc::setS($c);
    /** @mir-check $c is int */
    echo $c;
}
