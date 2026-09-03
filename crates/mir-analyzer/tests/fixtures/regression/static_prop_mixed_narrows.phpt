===description===
`is_int(self::$prop)` / `self::$prop === 42` / `self::$prop === 'x'` on an
untyped (mixed) static property now narrow it, mirroring the instance-property
siblings — the static-prop helpers had a leftover `is_mixed()` early return
these already had removed.
===config===
suppress=MissingConstructor,MissingPropertyType
===file===
<?php
class Box {
    public static $value;
}

function isIntNarrowsMixedStaticProp(): void {
    if (is_int(Box::$value)) {
        /** @mir-check Box::$value is int */
        $_ = 1;
    }
}

function isIntFalseBranchLeavesMixedStaticProp(): void {
    if (!is_int(Box::$value)) {
        /** @mir-check Box::$value is mixed */
        $_ = 1;
    }
}

function literalIntNarrowsMixedStaticProp(): void {
    if (Box::$value === 42) {
        /** @mir-check Box::$value is 42 */
        $_ = 1;
    }
}

function literalStringNarrowsMixedStaticProp(): void {
    if (Box::$value === 'hello') {
        /** @mir-check Box::$value is 'hello' */
        $_ = 1;
    }
}
===expect===
