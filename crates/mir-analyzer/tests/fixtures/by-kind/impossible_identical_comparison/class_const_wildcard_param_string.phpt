===description===
String constants resolve through a wildcard and through an exact `self::NAME` / own-name reference.
===file===
<?php
class B {
    const KIND_A = 'a';
    const KIND_B = 'b';

    /** @param self::KIND_* $k */
    public function wild($k): bool {
        /** @mir-check $k is 'a'|'b' */
        return $k === self::KIND_B;
    }

    /** @param B::KIND_A $k */
    public function exact($k): bool {
        /** @mir-check $k is 'a' */
        return $k === self::KIND_A;
    }
}
===expect===
