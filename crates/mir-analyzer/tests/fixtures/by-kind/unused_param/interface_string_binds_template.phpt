===description===
interface-string<T> argument binds T to the named interface, not the bound
===config===
suppress=UnusedVariable
===file===
<?php
/** @template T */
class Wrapper {}

interface Shape {}
interface Polygon {}

/**
 * @template T of object
 * @param interface-string<T> $iface
 * @return Wrapper<T>
 */
function make(string $iface): Wrapper { return new Wrapper(); }

$shapeWrapper = make(Shape::class);
$polygonWrapper = make(Polygon::class);
/** @mir-check $shapeWrapper is Wrapper<Shape> */
/** @mir-check $polygonWrapper is Wrapper<Polygon> */
===expect===
UnusedParam@13:14-13:27: Parameter $iface is never used
