===description===
Calling a @pure static method from inside another @pure function is allowed.
===file===
<?php
class MathUtil {
    /** @pure */
    public static function square(int $x): int {
        return $x * $x;
    }
}

/** @pure */
function callIt(int $x): int {
    return MathUtil::square($x);
}
===expect===
