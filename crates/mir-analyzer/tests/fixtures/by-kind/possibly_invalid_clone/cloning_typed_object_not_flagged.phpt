===description===
PossiblyInvalidClone does NOT fire when the clone target is a pure object type.
===file===
<?php
class Config {}

function copy(Config $c): Config {
    return clone $c;
}
===expect===
