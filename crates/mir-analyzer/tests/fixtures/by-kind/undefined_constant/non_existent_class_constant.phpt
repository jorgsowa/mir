===description===
Non existent class constant
===file===
<?php
class Foo {}
/**
 * @return Foo::HELLO|5
 */
function getVal()
//       ^^^^^^ UndefinedDocblockClass: Docblock type 'Foo::HELLO' does not exist
{
    return 5;
}
===expect===
