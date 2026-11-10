===description===
`clone($obj, [...])` writes each literal key on the copy like a property assignment.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
final class Point
{
    public function __construct(
        public int $x,
        public ?string $label = null,
    ) {}
}

class Magic
{
    public function __set(string $name, mixed $value): void {}
}

function f(Point $p, Magic $m, int|string $u): int
{
    $a = clone($p, ['x' => 2, 'label' => null]);
    $b = clone($p, ['nope' => 1]);
    //             ^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Point::$nope is deprecated
    $c = clone($p, ['x' => 'str']);
    //             ^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $x expects 'int', cannot assign '"str"'
    $d = clone($p, ['x' => $u]);
    //             ^^^^^^^^^^^ PossiblyInvalidPropertyAssignmentValue: Property $x expects 'int', the assigned 'int|string' might not match
    $e = clone($m, ['any' => 1]);
    var_dump($e);

    return $a->x + $b->x + $c->x + $d->x;
}
