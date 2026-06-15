===description===
Wrong case class name in parameter type hint is reported.
===config===
suppress=UnusedParam
===file===
<?php
class Request {}
function handle(request $r): void {}
===expect===
WrongCaseClass@3:16-3:23: Class name 'request' has incorrect casing; use 'Request'
