===description===
Non existent class constant
===file===
<?php
class Foo {}
/**
 * @return Foo::HELLO|5
 */
function getVal()
{
    return 5;
}
===expect===
