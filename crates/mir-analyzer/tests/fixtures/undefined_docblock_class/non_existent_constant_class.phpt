===description===
nonExistentConstantClass
===file===
<?php
/**
 * @return Foo::HELLO|5
 */
function getVal()
{
    return 5;
}
===expect===
UndefinedDocblockClass
===ignore===
TODO
