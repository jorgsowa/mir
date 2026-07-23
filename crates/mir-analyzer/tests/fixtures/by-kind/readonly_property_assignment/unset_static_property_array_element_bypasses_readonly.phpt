===description===
unset(self::$store[$k]) on a readonly static property bypassed readonly
enforcement the same way the plain static array-index-write case did
before it was fixed -- the unset chain-walk had no StaticPropertyAccess
arm at all.
===file===
<?php
class Registry {
    /** @readonly */
    public static array $store = [];

    public static function evict(string $k): void {
        unset(self::$store[$k]);
    }
}
===expect===
ReadonlyPropertyAssignment@7:14-7:30: Cannot assign to readonly property Registry::$store outside of constructor
