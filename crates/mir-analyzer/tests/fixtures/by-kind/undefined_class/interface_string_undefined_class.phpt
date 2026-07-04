===description===
Passing a non-existent class name to an interface-string parameter emits UndefinedClass
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @param interface-string $ifaceName
 */
function describe(string $ifaceName) {
    return $ifaceName;
}

describe("NonExistentInterface");
===expect===
UndefinedClass@9:9-9:31: Class NonExistentInterface does not exist
