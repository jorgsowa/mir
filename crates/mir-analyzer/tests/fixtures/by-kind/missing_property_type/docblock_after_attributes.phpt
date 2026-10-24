===description===
A property docblock written after its attributes types the property, as in PHP, and does not
move to the next property.
===file===
<?php
#[Attribute]
final class Column {}
final class Entity {
    #[Column]
    /** @var string */
    public $name = "";
    public $untyped;
//  ^^^^^^^^^^^^^^^ MissingPropertyType: Property Entity::$untyped has no type annotation
}
