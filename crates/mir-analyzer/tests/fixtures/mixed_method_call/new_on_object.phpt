===description===
New on object
===file===
<?php
function f(object $o): object
{
    return new $o;
}

===expect===
MixedMethodCall
===ignore===
TODO
