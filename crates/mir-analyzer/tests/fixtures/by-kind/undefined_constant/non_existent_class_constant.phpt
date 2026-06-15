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
UndefinedDocblockClass@6:9-6:15: Docblock type 'Foo::HELLO' does not exist
