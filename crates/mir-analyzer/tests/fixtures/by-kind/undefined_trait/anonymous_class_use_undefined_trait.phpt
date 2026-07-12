===description===
An anonymous class using a nonexistent trait must report UndefinedTrait,
matching a named class's `use` check.
===config===
suppress=UnusedVariable
===file===
<?php
$x = new class {
    use UndefinedTrait;
};
===expect===
UndefinedTrait@3:8-3:22: Trait UndefinedTrait does not exist
