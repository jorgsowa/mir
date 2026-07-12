===description===
Enum cannot be attribute class
===file===
<?php
#[Attribute]
enum Foo {
    case Bar;
}
===expect===
InvalidAttribute@2:2-2:11: Enums cannot be attribute classes
