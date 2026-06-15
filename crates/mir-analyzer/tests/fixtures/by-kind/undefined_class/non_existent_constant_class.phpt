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
UndefinedDocblockClass@5:9-5:15: Docblock type 'Foo::HELLO' does not exist
