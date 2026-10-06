===description===
Array keys of string keyed array doesnt conform to int list
===file===
<?php
/**
 * @return list<int>
 */
function getKeys() {
    return array_keys(["foo" => 42, "bar" => 42]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'non-empty-list<"foo"|"bar">' is not compatible with declared 'list<int>'
}
