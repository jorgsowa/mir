===description===
interface-string parameter accepts a matching interface reference (positive case)
===config===
suppress=MissingReturnType
===file===
<?php
interface Shape {
    public function area(): float;
}

/**
 * @param interface-string<Shape> $ifaceName
 */
function describe(string $ifaceName) {
    return $ifaceName;
}

describe(Shape::class);
===expect===
