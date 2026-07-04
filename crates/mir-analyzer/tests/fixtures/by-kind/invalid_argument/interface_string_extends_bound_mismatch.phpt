===description===
interface-string<Parent> does NOT satisfy a parameter typed interface-string<Child>
(the reverse of the covariant-safe direction) — Parent is not necessarily a Child.
===config===
suppress=MissingReturnType,UnusedVariable
===file===
<?php
interface Shape {}
interface Polygon extends Shape {}

/** @param interface-string<Polygon> $className */
function needsPolygon(string $className) {
    return $className;
}

function forward(string $shape): void {
    /** @var interface-string<Shape> $shape */
    needsPolygon($shape);
}
===expect===
InvalidArgument@12:17-12:23: Argument $className of needsPolygon() expects 'interface-string<Polygon>', got 'interface-string<Shape>'
