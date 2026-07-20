===description===
`str_contains(self::$prop, 'x')` / `str_starts_with(...)` / `str_ends_with(...)`
narrow a static property — ScalarArgTarget has no static-property variant
(tracked as S19), so these previously matched neither Var nor Prop on a
static receiver and narrowed nothing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Box {
    /** @var string */
    public static $value = '';
}

function strContainsNarrowsStaticProp(): void {
    if (str_contains(Box::$value, 'x')) {
        /** @mir-check Box::$value is non-empty-string */
        $_ = 1;
    }
}

function strStartsWithNarrowsStaticProp(): void {
    if (str_starts_with(Box::$value, 'x')) {
        /** @mir-check Box::$value is non-empty-string */
        $_ = 1;
    }
}

function strEndsWithNarrowsStaticProp(): void {
    if (str_ends_with(Box::$value, 'x')) {
        /** @mir-check Box::$value is non-empty-string */
        $_ = 1;
    }
}

function emptyNeedleDoesNotNarrowStaticProp(): void {
    // An empty needle is trivially "found" at offset 0 even in an empty
    // haystack — no narrowing should happen.
    if (str_contains(Box::$value, '')) {
        /** @mir-check Box::$value is string */
        $_ = 1;
    }
}
===expect===
