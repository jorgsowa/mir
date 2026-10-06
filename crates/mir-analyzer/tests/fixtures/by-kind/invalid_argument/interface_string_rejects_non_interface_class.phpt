===description===
Passing a concrete class name (that exists but is not an interface) to an
interface-string parameter emits NotAnInterface
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//       ^^^^^^^^^^^^^^^ NotAnInterface: ConcreteThing is not an interface
