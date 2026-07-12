===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum case.
===file===
<?php
enum Status {
    #[Cache]
    case Active;
}
===expect===
UndefinedAttributeClass@3:6-3:11: Attribute class Cache does not exist
