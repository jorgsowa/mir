===description===
Sibling of deprecated_class_with_new_attr: interface.rs only read the
docblock tag, missing the #[Deprecated] attribute fallback class.rs has.
===file===
<?php
#[\Deprecated]
//^^^^^^^^^^^ CompileError: Cannot apply #[\Deprecated] to interface Container [deprecated-on-invalid-target]
interface Container {}

class A implements Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface Container is deprecated
