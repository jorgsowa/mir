===description===
Before PHP 8.5, `#[Attribute]` on an abstract class, interface, trait or enum compiles; the class only fails when used as an attribute.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: Abstract classes cannot be attribute classes
abstract class A {}

#[Attribute]
//^^^^^^^^^ InvalidAttribute: Interfaces cannot be attribute classes
interface I {}

#[Attribute]
//^^^^^^^^^ InvalidAttribute: Traits cannot be attribute classes
trait T {}

#[Attribute]
//^^^^^^^^^ InvalidAttribute: Enums cannot be attribute classes
enum E {}
