===description===
Writing an undeclared property is deprecated from PHP 8.2, in every write form, unless the class has `__set` or `#[AllowDynamicProperties]` (inherited).
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
