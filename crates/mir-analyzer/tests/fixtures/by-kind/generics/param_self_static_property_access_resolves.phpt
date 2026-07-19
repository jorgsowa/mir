===description===
Property access through a `self`/`static`-typed *parameter* (not `$this`)
resolves the real property type — `resolve_property_type` only matched
`TNamedObject`, so a param stored as `TSelf`/`TStaticObject` (per
`@param self`/`@param static`) fell through to `mixed`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Point {
    public int $x = 0;

    /** @param self $other */
    public function distanceTo($other): int {
        /** @mir-check $other->x is int */
        $_ = 1;
        return $other->x;
    }

    /** @param static $other */
    public function sameXAs($other): int {
        /** @mir-check $other->x is int */
        $_ = 1;
        return $other->x;
    }
}
===expect===
