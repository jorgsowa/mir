===description===
Forbid template annotation on short closure
===file===
<?php
/** @template T */
fn(): bool => false;

===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @template annotations are not supported on closures or arrow functions
