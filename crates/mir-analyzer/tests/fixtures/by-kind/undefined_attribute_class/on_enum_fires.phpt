===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum declaration.
===file===
<?php
#[Cache]
//^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
enum Status {
    case Active;
}
