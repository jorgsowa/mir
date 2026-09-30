===description===
InvalidStaticInvocation fires independently for each non-static method called statically.
===file===
<?php
class Api {
    public function getUser(): string { return ""; }
    public function postUser(): string { return ""; }
}

Api::getUser();
//<^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Api::getUser() cannot be called statically
Api::postUser();
//<^^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Api::postUser() cannot be called statically
===expect===
