===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum case.
===file===
<?php
enum Status {
    #[Cache]
//    ^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
    case Active;
}
