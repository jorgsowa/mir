===description===
Missing class
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
UnusedPsalmSuppress@7:0-7:0: Suppress annotation for 'UndefinedDocblockClass' is never used
UnusedPsalmSuppress@10:0-10:0: Suppress annotation for 'UndefinedDocblockClass' is never used
