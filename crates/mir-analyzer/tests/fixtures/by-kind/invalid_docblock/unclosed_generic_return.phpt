===description===
unclosed generic return
===file===
<?php
/**
 * @return array<
// ^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @return has unclosed generic type `array<`
 */
function foo(): mixed { return []; }
//       ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
===expect===
