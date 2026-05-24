===description===
variantProperties
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
NonInvariantPropertyType
===ignore===
TODO
