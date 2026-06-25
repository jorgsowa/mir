===description===
InvalidStaticInvocation fires independently for each non-static method called statically.
===file===
<?php
class Api {
    public function getUser(): string { return ""; }
    public function postUser(): string { return ""; }
}

Api::getUser();
Api::postUser();
===expect===
InvalidStaticInvocation@7:0-7:14: Non-static method Api::getUser() cannot be called statically
InvalidStaticInvocation@8:0-8:15: Non-static method Api::postUser() cannot be called statically
