===description===
`resource` in a signature names a class; it is only reported when no such class is declared.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App {
    final class Resource {}
    function takesClass(Resource $r): void {}
    function takesLower(resource $r): void {}
}

namespace Other {
    function takesHandle(resource $r): void {}
//                       ^^^^^^^^ NativeResourceType: 'resource' is not a native type; PHP reads it as class 'Other\resource', which no resource satisfies
}
