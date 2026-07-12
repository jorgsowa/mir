===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an enum method.
===file===
<?php
enum Status {
    case Active;

    #[Cache]
    public function label(): string {
        return "active";
    }
}
===expect===
UndefinedAttributeClass@5:6-5:11: Attribute class Cache does not exist
