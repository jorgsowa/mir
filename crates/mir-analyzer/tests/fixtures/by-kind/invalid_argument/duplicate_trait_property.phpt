===description===
Duplicate trait property
===file===
<?php
trait T {
    public mixed $foo = 5;
    protected static mixed $foo;
}

===expect===
ParseError@4:5-4:32: Parse error: Cannot redeclare property $foo
