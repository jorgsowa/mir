===description===
missingClass
===file===
<?php
/**
 * @suppress UndefinedDocblockClass
 * @suppress InvalidReturnType
 * @return Editable
 */
function get() {}

/** @suppress UndefinedDocblockClass */
clone get();
===expect===
InvalidClone
===ignore===
TODO
