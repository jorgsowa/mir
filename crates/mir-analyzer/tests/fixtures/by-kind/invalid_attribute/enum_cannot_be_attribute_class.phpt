===description===
Enum cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: Enums cannot be attribute classes
enum Foo {
    case Bar;
}
