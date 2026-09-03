===description===
`strpos(self::$prop, 'x')` / `array_search(self::$prop, [...], true)`
narrow a static property used as the haystack/needle argument —
`ScalarArgTarget` has no static-property variant (tracked as S19), so
these previously matched neither Var nor Prop on a static receiver and
narrowed nothing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Box {
    /** @var string */
    public static $tag = '';

    /** @var string */
    public static $needle = '';
}

function strposNarrowsStaticProp(): void {
    if (strpos(Box::$tag, 'x') !== false) {
        /** @mir-check Box::$tag is non-empty-string */
        $_ = 1;
    }
}

function arraySearchNarrowsStaticProp(): void {
    if (array_search(Box::$needle, ['a', 'b'], true) !== false) {
        /** @mir-check Box::$needle is 'a'|'b' */
        $_ = 1;
    }
}
===expect===
