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
//       ^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class NonExistentInterface does not exist
===expect===
