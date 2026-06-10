===description===
No string allowed in key of int float string array
===ignore===
TODO
===file===
<?php
/**
 * @return key-of<array<int, string>|array<"42.0", string>>
 */
function getKey(bool $asInt) {
    if ($asInt) {
        return 42;
    }
    return "42";
}

===expect===
