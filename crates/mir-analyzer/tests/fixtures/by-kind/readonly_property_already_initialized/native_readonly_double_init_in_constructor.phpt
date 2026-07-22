===description===
A native `readonly` property assigned twice in its own constructor is a
runtime error ("Cannot modify readonly property ... once initialized") on
the second assignment — only the first write is legal.
===config===
suppress=UnusedParam
===file===
<?php
class Point {
    public readonly int $x;

    public function __construct(int $x) {
        $this->x = $x;
        $this->x = $x + 1;
    }
}
===expect===
ReadonlyPropertyAlreadyInitialized@7:8-7:25: Cannot modify readonly property Point::$x — already initialized
