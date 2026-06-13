===description===
InvalidClone does NOT fire when cloning a typed object.
===file===
<?php
class Config {}

function copy(Config $c): Config {
    return clone $c;
}
===expect===
