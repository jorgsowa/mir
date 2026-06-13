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
UndefinedDocblockClass@6:10-6:16: Docblock type 'Foo::HELLO' does not exist
