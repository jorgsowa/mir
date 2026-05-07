===description===
reports constructor argument shape
===file===
<?php
class User {
    public function __construct(string $name) {}
}
new User();
new User('Ada', 'Grace');
===expect===
UnusedParam@3:32: Parameter $name is never used
TooFewArguments@5:0: Too few arguments for User::__construct(): expected 1, got 0
TooManyArguments@6:16: Too many arguments for User::__construct(): expected 1, got 2
