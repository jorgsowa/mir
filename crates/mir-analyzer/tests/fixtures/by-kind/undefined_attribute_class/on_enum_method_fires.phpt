===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum method.
===file===
<?php
enum Status {
    case Active;

    #[Cache]
//    ^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
    public function label(): string {
        return "active";
    }
}
===expect===
