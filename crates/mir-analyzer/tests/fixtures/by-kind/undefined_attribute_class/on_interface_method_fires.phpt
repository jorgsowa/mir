===description===
UndefinedAttributeClass fires when an undefined attribute is placed on an interface method.
===file===
<?php
interface Repository {
    #[Cache]
    public function findAll(): array;
}
===expect===
UndefinedAttributeClass@3:6-3:11: Attribute class Cache does not exist
