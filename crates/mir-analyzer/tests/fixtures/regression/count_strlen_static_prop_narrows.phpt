===description===
`count(self::$prop) op N`/`strlen(self::$prop) op N` (both orderings, and
the equality operators) narrow a static property — `ScalarArgTarget` has no
static-property variant, so these previously matched neither Var nor Prop
on a static receiver and narrowed nothing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Bag {
    /** @var array */
    public static $items;
    /** @var string */
    public static $name;
}

function countGreaterThanNarrows(): void {
    if (count(Bag::$items) > 0) {
        /** @mir-check Bag::$items is non-empty-array */
        $_ = 1;
    }
}

function countOnRightSideNarrows(): void {
    if (0 < count(Bag::$items)) {
        /** @mir-check Bag::$items is non-empty-array */
        $_ = 1;
    }
}

function countEqualityNarrows(): void {
    if (count(Bag::$items) === 5) {
        /** @mir-check Bag::$items is non-empty-array */
        $_ = 1;
    }
}

function strlenGreaterThanNarrows(): void {
    if (strlen(Bag::$name) > 0) {
        /** @mir-check Bag::$name is non-empty-string */
        $_ = 1;
    }
}

function strlenEqualityNarrows(): void {
    if (strlen(Bag::$name) === 5) {
        /** @mir-check Bag::$name is non-empty-string */
        $_ = 1;
    }
}
===expect===
