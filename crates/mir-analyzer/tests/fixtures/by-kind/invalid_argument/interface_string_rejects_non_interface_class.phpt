===description===
Passing a concrete class name (that exists but is not an interface) to an
interface-string parameter emits NotAnInterface
===config===
suppress=MissingReturnType
===file===
<?php
class ConcreteThing {}

/**
 * @param interface-string $ifaceName
 */
function describe(string $ifaceName) {
    return $ifaceName;
}

describe("ConcreteThing");
===expect===
NotAnInterface@11:9-11:24: ConcreteThing is not an interface
