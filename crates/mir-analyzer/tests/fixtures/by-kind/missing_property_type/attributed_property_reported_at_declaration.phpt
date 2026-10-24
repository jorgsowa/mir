===description===
Attributed properties and promoted parameters are reported at the declaration, not at the attribute line.
===file===
<?php
#[Attribute]
final class Column {}
final class Entity {
    #[Column]
    public $id;
//  ^^^^^^^^^^ MissingPropertyType: Property Entity::$id has no type annotation
    public function __construct(
        #[Column]
        public $name = "",
//      ^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Entity::$name has no type annotation
    ) {}
}
