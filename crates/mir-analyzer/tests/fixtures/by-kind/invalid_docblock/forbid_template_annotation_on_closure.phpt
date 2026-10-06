===description===
Forbid template annotation on closure
===file===
<?php
/** @template T */
//  ^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @template annotations are not supported on closures or arrow functions
function (): void {};
