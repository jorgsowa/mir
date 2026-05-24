===description===
inexistentCallableinCallableString
===file===
<?php
/**
 * @param callable-string $c
 */
function c(string $c): void {
    $c();
}

c("hii");
===expect===
InvalidArgument
===ignore===
TODO
