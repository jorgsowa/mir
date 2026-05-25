===description===
Cannot name class constant class
===file===
<?php
class Foo
{
    /** @var class-string<Bar> */
    protected const CLASS = Bar::class;
}

class Bar {}

===expect===
ReservedWord
===ignore===
TODO
