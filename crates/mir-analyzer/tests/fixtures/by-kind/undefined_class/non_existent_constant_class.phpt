===description===
Non existent constant class
===file===
<?php
/**
 * @return Foo::HELLO|5
 */
function getVal()
//       ^^^^^^ UndefinedDocblockClass: Docblock type 'Foo::HELLO' does not exist
{
    return 5;
}
