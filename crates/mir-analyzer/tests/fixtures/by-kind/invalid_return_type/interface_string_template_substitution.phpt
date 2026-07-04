===description===
interface-string<T> in return type is substituted with the inferred interface
===config===
suppress=UnusedVariable
===file===
<?php
interface Shape {}
interface Polygon {}

/**
 * @template T of object
 * @param interface-string<T> $iface
 * @return interface-string<T>
 */
function identity(string $iface): string { return $iface; }

$shape = identity(Shape::class);
$polygon = identity(Polygon::class);
/** @mir-check $shape is interface-string<Shape> */
/** @mir-check $polygon is interface-string<Polygon> */
===expect===
