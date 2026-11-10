===description===
A native `resource` return type is a class name to PHP, not an undefined class: NativeResourceType.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function makeHandle(): resource {
//                     ^^^^^^^^ NativeResourceType: 'resource' is not a native type; PHP reads it as class 'resource', which no resource satisfies
    return fopen('php://memory', 'r');
}
