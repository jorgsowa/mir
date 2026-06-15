===description===
UndefinedDocblockClass fires when a @param docblock names a class that does
not exist, even without a native type hint.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param NonExistentParamClass $x
 */
function process($x): void {}

===expect===
UndefinedDocblockClass@5:9-5:16: Docblock type 'NonExistentParamClass' does not exist
