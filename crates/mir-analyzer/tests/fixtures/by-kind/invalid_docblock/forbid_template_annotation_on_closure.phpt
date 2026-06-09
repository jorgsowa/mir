===description===
Forbid template annotation on closure
===file===
<?php
/** @template T */
function (): void {};

===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @template annotations are not supported on closures or arrow functions
