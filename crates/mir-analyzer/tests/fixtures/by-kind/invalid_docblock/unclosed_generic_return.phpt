===description===
unclosed generic return
===file===
<?php
/**
 * @return array<
 */
function foo(): mixed { return []; }
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @return has unclosed generic type `array<`
UndefinedDocblockClass@5:10-5:13: Docblock type 'array<' does not exist
