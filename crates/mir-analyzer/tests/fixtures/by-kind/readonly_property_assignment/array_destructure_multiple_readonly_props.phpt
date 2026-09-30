===description===
`[$this->x, $this->y] = $vals;` writes to TWO readonly properties in one
statement, but the array-destructure arm passed the same outer statement
span to every element — the second violation collided with the first's
(kind, file, line, col_start) dedup key and was silently discarded.
===config===
suppress=MissingConstructor
===file===
<?php
class Point {
    /** @readonly */
    public float $x;
    /** @readonly */
    public float $y;

    public function reset(array $vals): void {
        [$this->x, $this->y] = $vals;
//       ^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Point::$x outside of constructor
//                 ^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Point::$y outside of constructor
    }
}
===expect===
