===description===
Sibling of deprecated_class_as_param: EnumDef had no deprecated field at
all, so @deprecated on an enum was silently dropped.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @deprecated use Status instead
 */
enum OldStatus { case A; case B; }

function foo(OldStatus $s): void {}
//           ^^^^^^^^^ DeprecatedClass: Class OldStatus is deprecated: use Status instead
===expect===
