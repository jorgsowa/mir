===description===
Legacy native `resource` in a parameter type position must not be treated as an
undefined class while mir reports the underlying declaration error.
===config===
suppress=UnusedFunction,UnusedParam
===file===
<?php
function takesResource(resource $value): void {}
===expect===

