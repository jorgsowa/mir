===description===
A write through a union whose classes all lack the property is checked against each of them.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Plain {}
enum Suit {
    case Hearts;
}

function enumSecond(Plain|Suit $u): void {
    $u->y = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Plain::$y is deprecated
//  ^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Suit::$y
    $u->y = 2;
}
