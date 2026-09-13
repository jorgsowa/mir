===description===
Legacy `resource` parameter types are not undefined classes.
===config===
suppress=UnusedFunction,UnusedParam
===file===
<?php
function takesResource(resource $value): void {}
===expect===
