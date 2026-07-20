===description===
`new Mid("wrong")` on a class that BOTH declares its own `@template U` AND
fixes an inherited ancestor's template via `@extends Base<int>` still
checks the inherited constructor arg against the fixed type — previously,
having ANY own template at all made the whole inherited-binding merge skip
entirely, silently accepting any argument type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
class Base {
    /** @param T $value */
    public function __construct($value) {}
}

/**
 * @template U
 * @extends Base<int>
 */
class Mid extends Base {
}

new Mid(5);
new Mid("wrong");
===expect===
InvalidArgument@16:8-16:15: Argument $value of Mid::__construct() expects 'int', got '"wrong"'
