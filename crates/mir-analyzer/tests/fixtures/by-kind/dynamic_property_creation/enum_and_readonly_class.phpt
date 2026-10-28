===description===
Enums and readonly classes throw `Cannot create dynamic property` instead of deprecating.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
enum Suit {
    case Hearts;

    public function mark(): void {
        $this->x = 1;
//      ^^^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Suit::$x
    }
}

readonly class Point {
    public function __construct(public int $x) {}
}

function writes(Suit $s, Point $p): void {
    $s->value = 1;
//  ^^^^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Suit::$value
    $p->y = 2;
//  ^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Point::$y
}
