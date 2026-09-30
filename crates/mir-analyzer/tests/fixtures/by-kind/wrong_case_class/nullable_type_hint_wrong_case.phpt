===description===
Wrong case class name in nullable type hint is reported.
===config===
suppress=UnusedParam
===file===
<?php
class User {}
function find(int $id): ?user { return null; }
//                       ^^^^ WrongCaseClass: Class name 'user' has incorrect casing; use 'User'
===expect===
