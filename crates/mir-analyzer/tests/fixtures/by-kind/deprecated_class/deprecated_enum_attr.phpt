===description===
Sibling of deprecated_enum_as_param: #[Deprecated] attribute fallback
(no docblock tag) on an enum declaration.
===config===
suppress=UnusedParam
===file===
<?php
#[\Deprecated]
enum OldStatus { case A; case B; }

function foo(OldStatus $s): void {}
===expect===
DeprecatedClass@5:13-5:22: Class OldStatus is deprecated
