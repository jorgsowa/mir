===description===
New on object
===file===
<?php
function f(object $o): object
{
    return new $o;
}

===expect===
InvalidStringClass@4:15-4:17: Dynamic class instantiation requires string or class-string type, got 'object'
