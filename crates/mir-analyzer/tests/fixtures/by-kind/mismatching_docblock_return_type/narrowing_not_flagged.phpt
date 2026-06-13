===description===
MismatchingDocblockReturnType does NOT fire when the docblock NARROWS the
native hint (more specific is fine — e.g. non-empty-string ⊆ string).
===file===
<?php
/** @return non-empty-string */
function greeting(): string { return "hello"; }

/** @return string */
function exact(): string { return "x"; }

===expect===
