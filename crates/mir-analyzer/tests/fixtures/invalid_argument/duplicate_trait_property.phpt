===description===
Duplicate trait property
===file===
<?php
trait T {
    public mixed $foo = 5;
    protected static mixed $foo;
}

===expect===
DuplicateProperty
===ignore===
TODO
