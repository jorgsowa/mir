===description===
Attribute target arg cannot be self const
===ignore===
TODO
===file===
<?php
#[Attribute(self::BAR)]
class Foo
{
    public const BAR = 1;
}

===expect===
