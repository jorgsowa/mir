===file:User.php===
<?php
class User {}
===file:Admin.php===
<?php
class Admin {}
===file:Service.php===
<?php
function createUser(User $u): void { var_dump($u); }
function test(): void {
    createUser(new Admin());
}
===expect===
Service.php: InvalidArgument: Argument $u of createUser() expects 'User', got 'Admin'
