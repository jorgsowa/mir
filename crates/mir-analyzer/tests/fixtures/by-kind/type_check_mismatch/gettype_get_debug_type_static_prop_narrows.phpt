===description===
`gettype(self::$prop) === 'literal'`/`get_debug_type(self::$prop) ===
'literal'` narrow a static property — `ScalarArgTarget` (the shared
Var/Prop extractor) has no static-property variant, so these previously
matched neither and narrowed nothing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Box {
    /** @var int|string */
    public static $value;
}

class Other {}

function gettypeNarrowsStaticProp(): void {
    if (gettype(Box::$value) === 'integer') {
        /** @mir-check Box::$value is int */
        $_ = 1;
    }
}

function getDebugTypeNarrowsStaticProp(): void {
    if (get_debug_type(Box::$value) === 'int') {
        /** @mir-check Box::$value is int */
        $_ = 1;
    }
}

function getDebugTypeExcludesStaticProp(): void {
    if (get_debug_type(Box::$value) !== 'int') {
        /** @mir-check Box::$value is string */
        $_ = 1;
    }
}

function looseComparisonAlsoNarrows(): void {
    if (gettype(Box::$value) == 'integer') {
        /** @mir-check Box::$value is int */
        $_ = 1;
    }
}

class ObjBox {
    /** @var Box|Other */
    public static $obj;
}

function getDebugTypeClassNameNarrowsStaticProp(): void {
    if (get_debug_type(ObjBox::$obj) === Box::class) {
        /** @mir-check ObjBox::$obj is Box */
        $_ = 1;
    }
}
===expect===
