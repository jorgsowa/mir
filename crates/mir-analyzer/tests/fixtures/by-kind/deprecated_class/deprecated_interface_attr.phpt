===description===
Sibling of deprecated_class_with_new_attr: interface.rs only read the
docblock tag, missing the #[Deprecated] attribute fallback class.rs has.
===file===
<?php
#[\Deprecated]
interface Container {}

class A implements Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface Container is deprecated
===expect===
