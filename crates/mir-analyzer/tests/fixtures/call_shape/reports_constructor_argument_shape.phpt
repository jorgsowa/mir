===file===
<?php
class User {
    public function __construct(string $name) {}
}
new User();
new User('Ada', 'Grace');
===expect===
UnusedParam: Parameter $name is never used
TooFewArguments: Too few arguments for User::__construct(): expected 1, got 0
TooManyArguments: Too many arguments for User::__construct(): expected 1, got 2
