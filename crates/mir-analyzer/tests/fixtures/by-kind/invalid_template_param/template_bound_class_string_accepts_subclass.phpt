===description===
A @template T of class-string<Shape> bound accepts a class-string naming a
subclass of Shape — codebase-aware `is_subtype` must walk the `extends`
hierarchy for class-string type params, not just check structural equality.
===config===
suppress=UnusedParam
===file===
<?php
class Shape {}
class Circle extends Shape {}

/**
 * @template T of class-string<Shape>
 * @param T $cls
 */
function make(string $cls): void {}

make(Circle::class);
===expect===
