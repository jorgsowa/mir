===description===
`new $class()` where `$class` holds a literal class name, with or without a
leading backslash, reports an interface, trait, enum or abstract class the
same way `new Name()` does. A `class-string<X>` may name any subclass of X,
so it isn't checked.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class Shape {}
trait Named {}
enum Suit { case Hearts; }

function literals(): void {
    $iface = 'Countable';
    new $iface();
//      ^^^^^^ InterfaceInstantiation: Cannot instantiate interface Countable
    $slashed = '\Countable';
    new $slashed();
//      ^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Countable
    $abstract = 'Shape';
    new $abstract();
//      ^^^^^^^^^ AbstractInstantiation: Cannot instantiate abstract class Shape
    $trait = 'Named';
    new $trait();
//      ^^^^^^ TraitInstantiation: Cannot instantiate trait Named
    $enum = 'Suit';
    new $enum();
//      ^^^^^ EnumInstantiation: Cannot instantiate enum Suit
    $concrete = '\DateTimeImmutable';
    $d = new $concrete();
}

/** @param class-string<Shape> $class */
function classString(string $class): void {
    $s = new $class();
}
