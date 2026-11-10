===description===
A native `resource` parameter type is a class name to PHP, not an undefined class: NativeResourceType.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesResource(resource $value): void {}
//                     ^^^^^^^^ NativeResourceType: 'resource' is not a native type; PHP reads it as class 'resource', which no resource satisfies
