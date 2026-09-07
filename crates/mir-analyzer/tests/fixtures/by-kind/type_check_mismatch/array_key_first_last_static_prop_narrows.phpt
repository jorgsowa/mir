===description===
`array_key_first(self::$prop) !== null` / `array_key_last(...) === null`
narrow a static property — `ScalarArgTarget` has no static-property
variant (tracked as S19), so these previously matched neither Var nor
Prop on a static receiver and narrowed nothing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Box {
    /** @var array */
    public static $items = [];
}

class Bag {
    /** @var array<string, int>|non-empty-array<string, int> */
    public static $items = [];
}

function firstNotNullNarrowsNonEmpty(): void {
    if (array_key_first(Box::$items) !== null) {
        /** @mir-check Box::$items is non-empty-array */
        $_ = 1;
    }
}

function lastIsNullNarrowsEmpty(): void {
    if (array_key_last(Bag::$items) === null) {
        /** @mir-check Bag::$items is array{} */
        $_ = 1;
    }
}
===expect===
