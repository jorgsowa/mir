===description===
Cannot name class constant class
===ignore===
TODO
===file===
<?php
class Foo
{
    /** @var class-string<Bar> */
    protected const CLASS = Bar::class;
}

class Bar {}

===expect===
