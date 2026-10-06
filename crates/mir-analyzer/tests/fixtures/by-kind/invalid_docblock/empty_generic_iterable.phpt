===description===
empty generic iterable return
===file===
<?php
/**
 * @return iterable<>
// ^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @return has empty generic type parameter in `iterable<>`
 */
function getData() { return []; }
===expect===
