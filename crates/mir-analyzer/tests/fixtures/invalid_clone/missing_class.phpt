===description===
missingClass
===file===
<?php
/**
 * @psalm-suppress UndefinedDocblockClass
 * @psalm-suppress InvalidReturnType
 * @return Editable
 */
function get() {}

/** @psalm-suppress UndefinedDocblockClass */
clone get();
===expect===
InvalidClone
===ignore===
TODO
