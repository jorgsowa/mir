===description===
Array keys of string array doesnt conforms to int list
===file===
<?php
/**
 * @param array<string, mixed> $array
 * @return list<int>
 */
function getKeys(array $array) {
    return array_keys($array);
}

===expect===
InvalidReturnType@7:5-7:31: Return type 'list<string>' is not compatible with declared 'list<int>'
