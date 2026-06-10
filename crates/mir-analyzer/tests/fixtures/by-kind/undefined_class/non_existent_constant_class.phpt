===description===
Non existent constant class
===ignore===
TODO
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
