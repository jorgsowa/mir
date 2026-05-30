===description===
does not report valid nested generic
===file===
<?php
/**
 * @return array<string, array<int>>
 */
function foo(): array { return []; }
===expect===
