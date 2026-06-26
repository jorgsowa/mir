===description===
PossiblyInvalidClone does NOT fire when cloning a union of objects (all atoms are object types).
===file===
<?php
class Logger {}
class Config {}
function f(Logger|Config $obj): void {
    clone $obj;
}
===expect===
