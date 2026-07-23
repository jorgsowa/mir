===description===
`self::$store['k'] = 1` mutates a readonly static property's contents just
as much as a plain `self::$store = ...` write does, but the static-property
array-index-write arm only ever checked purity (and only inside a @pure
function), never readonly.
===file===
<?php
class Registry {
    /** @readonly */
    public static array $store = [];

    public static function put(string $k, int $v): void {
        self::$store[$k] = $v;
    }
}
===expect===
ReadonlyPropertyAssignment@7:8-7:29: Cannot assign to readonly property Registry::$store outside of constructor
