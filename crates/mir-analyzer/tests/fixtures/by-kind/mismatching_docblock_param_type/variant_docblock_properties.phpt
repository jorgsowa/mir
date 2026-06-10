===description===
Variant docblock properties
===file===
<?php
class ParentClass
{
    /** @var null|string */
    protected $mightExist;
}

class ChildClass extends ParentClass
{
    /** @var string */
    protected $mightExist = "";
}
===expect===
