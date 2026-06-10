===description===
Variant properties
===ignore===
TODO
===file===
<?php
class ParentClass
{
    protected ?string $mightExist = null;
}

class ChildClass extends ParentClass
{
    protected string $mightExist = "";
}
===expect===
