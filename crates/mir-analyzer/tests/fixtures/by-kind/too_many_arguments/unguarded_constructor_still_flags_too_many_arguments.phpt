===description===
Without an existence guard, `new` on a class with no constructor is
checked normally and TooManyArguments is reported.
===config===
suppress=MissingReturnType
===file===
<?php
class NewApi {}

function check(): void {
    new NewApi(1);
//  ^^^^^^^^^^^^^ TooManyArguments: Too many arguments for NewApi::__construct(): expected 0, got 1
}
===expect===
