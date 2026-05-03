===description===
cross file wrong class passed
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
Service.php: InvalidArgument@4:15: Argument $u of createUser() expects 'User', got 'Admin'
===ignore===
TODO
