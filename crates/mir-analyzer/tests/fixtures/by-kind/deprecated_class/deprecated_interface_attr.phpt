===description===
Sibling of deprecated_class_with_new_attr: interface.rs only read the
docblock tag, missing the #[Deprecated] attribute fallback class.rs has.
===file===
<?php
#[\Deprecated]
interface Container {}

class A implements Container {}
===expect===
DeprecatedInterface@5:0-5:31: Interface Container is deprecated
