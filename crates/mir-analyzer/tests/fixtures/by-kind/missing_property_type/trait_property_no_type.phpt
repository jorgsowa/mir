===description===
MissingPropertyType fires for trait properties that have no type declaration.
===file===
<?php
trait HasName {
    public $name;
    protected $description;
}
===expect===
MissingPropertyType@3:4-3:16: Property HasName::$name has no type annotation
MissingPropertyType@4:4-4:26: Property HasName::$description has no type annotation
