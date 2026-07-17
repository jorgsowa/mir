===description===
`@mir-check EXPR is TYPE` now parses EXPR as a real PHP expression and runs
it through the same inference every other expression goes through, instead
of only supporting a bare `$variable` name. Property access, array/shape
keys, and static property access are all checkable directly, with no
intermediate `$x = EXPR;` assignment needed.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php

class Holder {
    public bool|string $flag = true;
}

function checksPropertyDirectly(Holder $h): void {
    if ($h->flag === true) {
        /** @mir-check $h->flag is true */
        $_ = null;
    }
}

/**
 * @param array{name: string, age?: int} $arr
 */
function checksArrayShapeKeyDirectly(array $arr): void {
    if (isset($arr['age'])) {
        /** @mir-check $arr['age'] is int */
        $_ = null;
    }
}

class Service {
    protected static ?string $name = null;

    public static function checksStaticPropDirectly(): void {
        if (self::$name !== null) {
            /** @mir-check self::$name is string */
            $_ = null;
        }
    }
}
===expect===
