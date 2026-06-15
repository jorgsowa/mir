===description===
No string allowed in key of int float string array
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
UndefinedDocblockClass@5:9-5:15: Docblock type 'key-of' does not exist
