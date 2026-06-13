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
UndefinedDocblockClass@3:10-3:20: Docblock type 'GhostClass' does not exist
