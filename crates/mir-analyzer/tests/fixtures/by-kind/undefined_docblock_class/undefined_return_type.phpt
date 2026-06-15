===description===
UndefinedDocblockClass fires when the @return docblock names a class that
does not exist anywhere in the codebase.
===file===
<?php
/** @return NonExistentReturnClass */
function missing(): mixed {
    return null;
}

===expect===
UndefinedDocblockClass@3:9-3:16: Docblock type 'NonExistentReturnClass' does not exist
