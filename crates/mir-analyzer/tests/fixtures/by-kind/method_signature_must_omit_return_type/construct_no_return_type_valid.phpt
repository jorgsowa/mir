===description===
__construct() without a return type is valid — no ParseError.
===file===
<?php
class A
{
    public function __construct()
    {
    }
}
===expect===
