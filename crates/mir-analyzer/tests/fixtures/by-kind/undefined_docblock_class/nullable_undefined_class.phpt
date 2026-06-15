===description===
UndefinedDocblockClass fires for nullable types too when the class inside
the union does not exist.
===file===
<?php
/** @return GhostClass|null */
function maybeGhost(): mixed {
    return null;
}

===expect===
UndefinedDocblockClass@3:9-3:19: Docblock type 'GhostClass' does not exist
