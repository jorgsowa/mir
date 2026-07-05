===description===
interface_exists($x) narrowing a known class-string<T> keeps the class name,
producing interface-string<T> — not a bare, unparameterized interface-string
that would then fail to satisfy a interface-string<T>-typed parameter.
===config===
suppress=UnusedParam
===file===
<?php
interface Shape {}

/** @param interface-string<Shape> $x */
function needsShapeIface(string $x): void {}

/** @param class-string<Shape> $className */
function test(string $className): void {
    if (interface_exists($className)) {
        /** @mir-check $className is interface-string<Shape> */
        needsShapeIface($className);
    }
}
===expect===
