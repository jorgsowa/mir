===description===
unclosed generic return
===file===
<?php
/**
 * @return array<
// ^^^^^^^^^^^^^^ InvalidDocblock: @return has unclosed generic type `array<`
 */
function foo(): mixed { return []; }
//       ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
