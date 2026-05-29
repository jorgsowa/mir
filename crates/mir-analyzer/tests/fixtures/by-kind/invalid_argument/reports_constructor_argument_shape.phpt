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
TooFewArguments@5:1-5:11: Too few arguments for User::__construct(): expected 1, got 0
TooManyArguments@6:17-6:24: Too many arguments for User::__construct(): expected 1, got 2
