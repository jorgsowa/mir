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
new User('Ada', 'Grace');
===expect===
TooFewArguments@5:0-5:10: Too few arguments for User::__construct(): expected 1, got 0
TooManyArguments@6:16-6:23: Too many arguments for User::__construct(): expected 1, got 2
