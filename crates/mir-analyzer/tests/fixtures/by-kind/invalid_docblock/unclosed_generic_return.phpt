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
InvalidDocblock@3:3-3:17: Invalid docblock: @return has unclosed generic type `array<`
