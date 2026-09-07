===description===
A conditional return type's (`@return ($x is T ? A : B)`) discriminator
parameter was resolved by its DECLARED index directly into arg_types
(built in call-site TEXTUAL order) -- silently reading the wrong
argument's type whenever a named argument reordered the call. Same bug
class as the already-fixed positional File/Unserialize sink check, never
mirrored for conditional returns. Covers all three call sites (function,
method, static method) since each shares the identical fix.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingThrowsDocblock
===file===
<?php
/**
 * @param bool $flag
 * @param mixed $value
 * @return ($flag is true ? int : string)
 */
function f(bool $flag, $value) {
    throw new \Exception();
}

$r1 = f(value: 'x', flag: true);
/** @mir-check $r1 is int */
echo "ok";

class Picker {
    /**
     * @param bool $flag
     * @param mixed $value
     * @return ($flag is true ? int : string)
     */
    public function pick(bool $flag, $value) {
        throw new \Exception();
    }

    /**
     * @param bool $flag
     * @param mixed $value
     * @return ($flag is true ? int : string)
     */
    public static function pickStatic(bool $flag, $value) {
        throw new \Exception();
    }
}

$p = new Picker();
$r2 = $p->pick(value: 'x', flag: true);
/** @mir-check $r2 is int */
echo "ok";

$r3 = Picker::pickStatic(value: 'x', flag: true);
/** @mir-check $r3 is int */
echo "ok";
===expect===
