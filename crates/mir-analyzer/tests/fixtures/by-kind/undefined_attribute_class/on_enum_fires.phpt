===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum declaration.
===file===
<?php
#[Cache]
enum Status {
    case Active;
}
===expect===
UndefinedAttributeClass@2:2-2:7: Attribute class Cache does not exist
