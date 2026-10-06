===description===
Forbid template annotation on short closure
===file===
<?php
/** @template T */
//  ^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @template annotations are not supported on closures or arrow functions
fn(): bool => false;

===expect===
