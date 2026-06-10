===description===
Attribute target arg cannot be self const
===file===
<?php
#[Attribute(self::BAR)]
class Foo
{
    public const BAR = 1;
}

===expect===
