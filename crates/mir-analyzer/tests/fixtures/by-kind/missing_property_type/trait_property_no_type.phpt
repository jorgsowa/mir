===description===
MissingPropertyType fires for trait properties that have no type declaration.
===file===
<?php
trait HasName {
    public $name;
//  ^^^^^^^^^^^^ MissingPropertyType: Property HasName::$name has no type annotation
    protected $description;
//  ^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property HasName::$description has no type annotation
}
