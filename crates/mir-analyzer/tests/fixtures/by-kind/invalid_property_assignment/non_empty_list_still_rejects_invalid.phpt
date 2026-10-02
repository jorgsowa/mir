===description===
non-empty-list<Parent> still rejects empty, unrelated, and possibly-empty values
===config===
suppress=UnusedVariable,MissingConstructor
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
    $c->shapes = [new Circle(), new Other()];
    $c->shapes = ['a' => new Circle()];
}
===expect===
InvalidPropertyAssignment@12:4-12:19: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{}'
InvalidPropertyAssignment@13:4-13:44: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{0: Circle, 1: Other}'
InvalidPropertyAssignment@14:4-14:38: Property $shapes expects 'non-empty-list<Shape>', cannot assign 'array{'a': Circle}'
