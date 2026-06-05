===description===
Wrong case class name in parameter type hint is reported.
===file===
<?php
class Request {}
function handle(request $r): void {}
===expect===
WrongCaseClass@3:17-3:24: Class name 'request' has incorrect casing; use 'Request'
