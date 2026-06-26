===description===
MissingPropertyType fires for abstract class properties that have no type declaration.
===file===
<?php
abstract class Entity {
    public $id;
    protected $payload;
}
===expect===
MissingPropertyType@3:4-3:14: Property Entity::$id has no type annotation
MissingPropertyType@4:4-4:22: Property Entity::$payload has no type annotation
