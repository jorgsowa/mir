===description===
non-empty-list<Parent> still rejects empty, unrelated, and possibly-empty values
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class Shape {}
final class Circle extends Shape {}
final class Other {}

class Canvas {
    /** @var non-empty-list<Shape> */
    public array $shapes;
}

function run(Canvas $c): void {
    $c->shapes = [];
//  ^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{}'
    $c->shapes = [new Circle(), new Other()];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{0: Circle, 1: Other}'
    $c->shapes = ['a' => new Circle()];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{'a': Circle}'
}
