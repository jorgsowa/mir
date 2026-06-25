===description===
self::CONST and parent::CONST expressions resolve to their literal types.
Integer, string, float, bool, and null constants are all inferred from their
initializer values, not widened to mixed.
===config===
suppress=UnusedVariable
===file===
<?php
class Consts {
    const INT_VAL    = 42;
    const STR_VAL    = 'hello';
    const FLOAT_VAL  = 3.14;
    const BOOL_TRUE  = true;
    const BOOL_FALSE = false;
    const NULL_VAL   = null;
    const NEG_INT    = -7;

    public function check(): void {
        $i = self::INT_VAL;
        /** @mir-check $i is 42 */
        $_ = $i;

        $s = self::STR_VAL;
        /** @mir-check $s is 'hello' */
        $_ = $s;

        $f = self::FLOAT_VAL;
        /** @mir-check $f is float */
        $_ = $f;

        $bt = self::BOOL_TRUE;
        /** @mir-check $bt is bool */
        $_ = $bt;

        $bf = self::BOOL_FALSE;
        /** @mir-check $bf is bool */
        $_ = $bf;

        $n = self::NULL_VAL;
        /** @mir-check $n is null */
        $_ = $n;

        $neg = self::NEG_INT;
        /** @mir-check $neg is -7 */
        $_ = $neg;
    }
}
===expect===
