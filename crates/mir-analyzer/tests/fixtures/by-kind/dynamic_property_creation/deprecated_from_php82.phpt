===description===
Writing an undeclared property is deprecated from PHP 8.2, in every write form (also `[]=` and `&`, which read the property first), unless the class has `__set` or `#[AllowDynamicProperties]` (inherited).
===config===
<mir>
  <phpVersion>8.2</phpVersion>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Plain {}
class OnlyGetter { public function __get(string $n): int { return 1; } }

function writes(Plain $a, OnlyGetter $g, Exception $e): void {
    $a->x = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$x is deprecated
    $a->y ??= 1;
//  ^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$y is deprecated
    [$a->w] = [1];
//   ^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$w is deprecated
    foreach ([1] as $a->v) {}
//                  ^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$v is deprecated
    $g->x = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property OnlyGetter::$x is deprecated
    $e->x = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Exception::$x is deprecated
}

function createdOnce(Plain $a): string {
    $a->x = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$x is deprecated
    $a->x = 2;
    $a->x .= '';
    return $a->x;
}

class InMethod {
    public function f(): void {
        $this->later = 1;
//      ^^^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property InMethod::$later is deprecated
    }
}

function writesThroughRead(Plain $a): void {
    $a->list[] = 1;
//  ^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$list is deprecated
    $a->list[] = 2;
    $a->map['k'] = 1;
//  ^^^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$map is deprecated
    $ref = &$a->z;
//  ^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$z is deprecated
}
