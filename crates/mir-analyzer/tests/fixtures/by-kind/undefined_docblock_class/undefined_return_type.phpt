===description===
UndefinedDocblockClass fires when the @return docblock names a class that
does not exist anywhere in the codebase.
===file===
<?php
/** @return NonExistentReturnClass */
function missing(): mixed {
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentReturnClass' does not exist
    return null;
}
