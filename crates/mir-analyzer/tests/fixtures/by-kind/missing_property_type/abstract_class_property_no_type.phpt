===description===
MissingPropertyType fires for abstract class properties that have no type declaration.
===file===
<?php
abstract class Entity {
    public $id;
//  ^^^^^^^^^^ MissingPropertyType: Property Entity::$id has no type annotation
    protected $payload;
//  ^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Entity::$payload has no type annotation
}
