===description===
Forbid template annotation on short closure
===file===
<?php
/** @template T */
fn(): bool => false;

===expect===
