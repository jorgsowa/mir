===description===
UndefinedDocblockClass fires for the element class of an `array<int, Foo>` @return docblock shape, not just a bare class name.
===file===
<?php
/** @return array<int, NonExistentElement> */
function missing(): array {
    return [];
}

===expect===
UndefinedDocblockClass@3:9-3:16: Docblock type 'NonExistentElement' does not exist
