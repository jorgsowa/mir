===description===
reports constructor argument shape
===config===
suppress=UnusedParam
===file===
<?php
class User {
    public function __construct(string $name) {}
}
new User();
//<^^^^^^^^^^ TooFewArguments: Too few arguments for User::__construct(): expected 1, got 0
new User('Ada', 'Grace');
//              ^^^^^^^ TooManyArguments: Too many arguments for User::__construct(): expected 1, got 2
===expect===
