===description===
reports constructor argument shape
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
