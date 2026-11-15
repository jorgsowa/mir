===description===
Marked parameters, object-typed tokens, names that aren't secrets, and
bodiless declarations are fine.
===config===
<mir>
  <issueHandlers>
    <MissingSensitiveParameter errorLevel="warning"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function login(string $user, #[\SensitiveParameter] string $password): bool { return true; }

function lex(PhpToken $token, int $tokenCount, string $passwordPolicy, string $tokenType): void {}

interface Authenticator {
    public function check(string $password): bool;
}

abstract class Base {
    abstract public function verify(string $secret): bool;
}
