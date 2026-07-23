===description===
A trailing free-text explanation after a named suppression's kind
(`@psalm-suppress UndefinedClass because of a vendor stub`) used to have
every subsequent word ("because", "of", "a", "vendor", "stub") parsed as
its own bogus additional kind name, each producing its own spurious
UnusedSuppress since none of them ever match a real issue kind. The real
kind (UndefinedClass) IS used here, so no UnusedSuppress should fire at all.
===config===
===file===
<?php
/** @psalm-suppress UndefinedClass because of a vendor stub */
new NoSuchClass();
===expect===
