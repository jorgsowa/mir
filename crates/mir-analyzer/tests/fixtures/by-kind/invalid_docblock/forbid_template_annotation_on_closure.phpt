===description===
Forbid template annotation on closure
===file===
<?php
/** @template T */
function (): void {};

===expect===
InvalidDocblock
===ignore===
TODO
