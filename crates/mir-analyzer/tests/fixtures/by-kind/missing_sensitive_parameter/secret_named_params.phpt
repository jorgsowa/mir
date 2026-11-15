===description===
Parameters named like a password, secret, token or API key, untyped or holding a
string, need `#[\SensitiveParameter]` to stay out of stack traces.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function login(string $user, string $password): bool { return true; }
//                           ^^^^^^^^^^^^^^^^ MissingSensitiveParameter: Parameter $password looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs

final class Client {
    public function __construct(private ?string $api_key = null) {}
//                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingSensitiveParameter: Parameter $api_key looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs
    public function refresh($accessToken): void {}
//                          ^^^^^^^^^^^^ MissingSensitiveParameter: Parameter $accessToken looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs
}

trait Signs {
    public function sign(string $payload, string|null $clientSecret): string { return $payload; }
//                                        ^^^^^^^^^^^^^^^^^^^^^^^^^ MissingSensitiveParameter: Parameter $clientSecret looks like a secret but lacks #[\SensitiveParameter], so its value can appear in stack traces and logs
}
