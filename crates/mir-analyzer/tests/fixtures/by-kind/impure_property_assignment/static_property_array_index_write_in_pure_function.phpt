===description===
self::$items[$k] = x; (an array-index write through a static-property
base) bypassed purity entirely -- the array-write loop's catch-all arm
for a non-variable, non-instance-property base only ever analyzed it as
a read, never reaching the ImpureStaticPropertyAssignment check the
plain `self::$items = x` form already has.
===file===
<?php
class Registry {
    public static array $items = [];

    /** @pure */
    public static function corrupt(): void {
        self::$items['x'] = 1;
    }
}
===expect===
ImpureStaticPropertyAssignment@7:8-7:29: Assigning to static property Registry::$items in a @pure function
