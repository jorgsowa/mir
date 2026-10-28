===description===
Classes that allow dynamic properties: `__set`, `#[AllowDynamicProperties]` on the class or a parent, `stdClass` and its subclasses, `@property` tags; a union where another class declares the property.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
#[AllowDynamicProperties]
class Allowed {}
class AllowedChild extends Allowed {}
class Magic { public function __set(string $n, mixed $v): void {} }
class Bag extends stdClass {}
/** @property int $p */
class Tagged {}
class Plain {}

function writes(Allowed $a, AllowedChild $c, Magic $m, Bag $b, stdClass $s, Tagged $t): void {
    $a->x = 1;
    $c->x = 1;
    $m->x = 1;
    $b->x = 1;
    $s->x = 1;
    $t->p = 1;
}

class Declares { public int $x = 0; }

function union(Plain|Declares $u): void {
    $u->x = 1;
}
