===description===
unclosed generic return
===file===
<?php
/**
 * @return array<
 */
function foo(): mixed { return []; }
//       ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @return has unclosed generic type `array<`
