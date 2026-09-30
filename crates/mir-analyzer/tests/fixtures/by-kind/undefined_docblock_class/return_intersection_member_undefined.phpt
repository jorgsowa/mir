===description===
UndefinedDocblockClass fires for a member of a `Foo&Bar` intersection @return docblock type.
===config===
suppress=InvalidReturnType
===file===
<?php
interface Countable2 {}
/** @return Countable2&NonExistentMember */
function missing(): mixed {
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentMember' does not exist
    return null;
}

===expect===
