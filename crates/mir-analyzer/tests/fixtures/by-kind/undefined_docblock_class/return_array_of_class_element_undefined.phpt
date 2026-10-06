===description===
UndefinedDocblockClass fires for the element class of an `array<int, Foo>` @return docblock shape, not just a bare class name.
===file===
<?php
/** @return array<int, NonExistentElement> */
function missing(): array {
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentElement' does not exist
    return [];
}
