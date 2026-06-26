===description===
PossiblyInvalidClone does NOT fire after a null guard narrows a nullable object to its object type.
===file===
<?php
class Config {}
function f(?Config $c): void {
    if ($c !== null) {
        clone $c;
    }
}
===expect===
