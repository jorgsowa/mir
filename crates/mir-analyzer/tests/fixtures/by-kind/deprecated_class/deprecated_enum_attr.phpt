===description===
Sibling of deprecated_enum_as_param: #[Deprecated] attribute fallback
(no docblock tag) on an enum declaration.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[\Deprecated]
//^^^^^^^^^^^ CompileError: Cannot apply #[\Deprecated] to enum OldStatus [deprecated-on-invalid-target]
enum OldStatus { case A; case B; }

function foo(OldStatus $s): void {}
//           ^^^^^^^^^ DeprecatedClass: Class OldStatus is deprecated
