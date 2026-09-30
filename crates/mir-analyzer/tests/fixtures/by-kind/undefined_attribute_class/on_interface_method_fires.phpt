===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an interface method.
===file===
<?php
interface Repository {
    #[Cache]
//    ^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
    public function findAll(): array;
}
===expect===
