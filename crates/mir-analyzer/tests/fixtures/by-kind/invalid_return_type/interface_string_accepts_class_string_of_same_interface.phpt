===description===
A declared `@return interface-string<Shape>` accepts `Shape::class`, which
types as class-string<Shape> (interfaces don't get their own ::class type) —
the name still literally resolves to an interface, so it satisfies the
narrower declared return type.
===config===
===file===
<?php
interface Shape {}

/** @return interface-string<Shape> */
function getShapeClass(): string {
    return Shape::class;
}
===expect===
