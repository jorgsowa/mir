===description===
Non existent constant class
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
UndefinedDocblockClass@5:10-5:16: Docblock type 'Foo::HELLO' does not exist
