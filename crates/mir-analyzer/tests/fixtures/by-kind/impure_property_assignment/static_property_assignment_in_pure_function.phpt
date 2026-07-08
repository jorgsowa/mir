===description===
Assigning to a class static property inside a @pure function must be
flagged — a static property IS the shared external state, same as a
global variable, unlike an instance property (only impure through a
specific receiver). The sibling instance-property check never covered
StaticPropertyAccess at all.
===file===
<?php
class Counter {
    public static int $count = 0;

    /** @pure */
    public static function bump(): void {
        self::$count = 5;
    }
}
===expect===
ImpureStaticPropertyAssignment@7:8-7:24: Assigning to static property Counter::$count in a @pure function
