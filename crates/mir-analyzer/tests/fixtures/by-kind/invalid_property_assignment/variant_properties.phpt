===description===
Variant properties — narrowing a nullable parent property type is a PHP fatal error
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
PropertyTypeRedeclarationMismatch@9:4-9:38: Type of ChildClass::$mightExist must be string|null (as in parent class), string given
