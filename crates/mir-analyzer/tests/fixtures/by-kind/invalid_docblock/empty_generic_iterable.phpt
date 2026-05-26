===description===
empty generic iterable return
===file===
<?php
/**
 * @return iterable<>
 */
function getData() { return []; }
===expect===
InvalidDocblock@2:0: Invalid docblock: @return has empty generic type parameter in `iterable<>`
