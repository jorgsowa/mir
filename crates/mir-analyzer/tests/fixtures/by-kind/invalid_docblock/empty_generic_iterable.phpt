===description===
empty generic iterable return
===file===
<?php
/**
 * @return iterable<>
 */
function getData() { return []; }
===expect===
InvalidDocblock@3:3-3:21: Invalid docblock: @return has empty generic type parameter in `iterable<>`
