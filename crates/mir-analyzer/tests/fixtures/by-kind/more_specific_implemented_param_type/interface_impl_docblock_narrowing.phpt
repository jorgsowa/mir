===description===
Implementing an interface method with a more-specific @param docblock must not
emit MethodSignatureMismatch. Docblock narrowing is an intentional refinement,
not an LSP violation — the native type hint is unchanged.
===config===
suppress=UnusedParam
===file===
<?php
class Shape {}
class Circle extends Shape {}
class Square extends Shape {}

interface Renderer {
    public function draw(Shape $shape): void;
}

class CircleRenderer implements Renderer {
    /** @param Circle $shape */
    public function draw(Shape $shape): void {}
}

class SquareRenderer implements Renderer {
    /** @param Square $shape */
    public function draw(Shape $shape): void {}
}
===expect===
