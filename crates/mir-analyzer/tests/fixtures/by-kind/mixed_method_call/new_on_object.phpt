===description===
New on object
===file===
<?php
function f(object $o): object
{
    return new $o;
//             ^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'object'
}
