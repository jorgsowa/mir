===description===
empty generic iterable return
===file===
<?php
/**
 * @return iterable<>
// ^^^^^^^^^^^^^^^^^^ InvalidDocblock: @return has empty generic type parameter in `iterable<>`
 */
function getData() { return []; }
