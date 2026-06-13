===description===
Wrong case class name in nullable type hint is reported.
===config===
suppress=UnusedParam
===file===
<?php
class User {}
function find(int $id): ?user { return null; }
===expect===
WrongCaseClass@3:26-3:30: Class name 'user' has incorrect casing; use 'User'
