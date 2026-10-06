===description===
non-empty-list<Parent> accepts list literals of subclasses on every assignment path
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class Shape {}
final class Circle extends Shape {}
final class Square extends Shape {}

class Canvas {
    /** @var non-empty-list<Shape> */
    public array $shapes;
    /** @var non-empty-list<Shape>|null */
    public ?array $maybe = null;
    /** @var non-empty-list<Shape> */
    public static array $registry;

    /** @param non-empty-list<Shape> $shapes */
    public function __construct(array $shapes) { $this->shapes = $shapes; }
//                  ^^^^^^^^^^^ PropertyPossiblyUninitialized: Property Canvas::$registry may be left uninitialized by the constructor
}

function run(Canvas $c): void {
    $c->shapes = [new Circle(), new Square()];
    $c->shapes = [new Circle(), new Circle()];
    $c->maybe = [new Circle(), new Square()];
    Canvas::$registry = [new Square(), new Circle()];
    $local = [new Circle(), new Square()];
    /** @mir-check $local is array{0: Circle, 1: Square} */
    $c->shapes = $local;
}
new Canvas([new Circle(), new Square()]);
